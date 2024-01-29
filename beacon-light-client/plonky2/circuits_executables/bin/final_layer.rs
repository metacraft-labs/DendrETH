use colored::Colorize;
use std::{println, time::Instant};

use anyhow::Result;
use circuits::build_final_circuit::build_final_circuit;
use circuits_executables::{
    crud::{
        fetch_final_layer_input, fetch_proof, fetch_proof_balances, load_circuit_data,
        save_final_proof, BalanceProof, ValidatorProof,
    },
    provers::SetPWValues,
};
use clap::{App, Arg};
use futures_lite::future;
use num::BigUint;
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};
use std::ops::Div;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let matches = App::new("")
        .arg(
            Arg::with_name("redis_connection")
                .short('r')
                .long("redis")
                .value_name("Redis Connection")
                .help("Sets a custom Redis connection")
                .takes_value(true)
                .default_value("redis://127.0.0.1:6379/"),
        )
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!(
        "{}",
        format!("Redis connection took: {:?}", elapsed).yellow()
    );

    let balance_data = load_circuit_data("37").unwrap();
    let commitment_data = load_circuit_data("commitment_mapper_40").unwrap();

    let (circuit_targets, circuit_data) = build_final_circuit(&balance_data, &commitment_data);

    let final_input_data = fetch_final_layer_input(&mut con).await?;

    let mut pw: PartialWitness<GoldilocksField> = PartialWitness::new();

    circuit_targets.set_pw_values(&mut pw, &final_input_data);

    let balance_proof: BalanceProof = fetch_proof_balances(&mut con, 37, 0).await?;

    let balance_final_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_proof.proof,
            &balance_data.common,
        )?;

    pw.set_proof_with_pis_target(
        &circuit_targets.balance_circuit_targets.proof,
        &balance_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .balance_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &balance_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .balance_circuit_targets
            .verifier_circuit_target
            .circuit_digest,
        balance_data.verifier_only.circuit_digest,
    );

    let epoch = BigUint::div(final_input_data.slot, 32u32).to_u64().unwrap();
    let commitment_proof: ValidatorProof = fetch_proof(&mut con, 0, epoch).await?;

    let commitment_final_proof = ProofWithPublicInputs::<
        GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >::from_bytes(commitment_proof.proof, &commitment_data.common)?;

    pw.set_proof_with_pis_target(
        &circuit_targets.commitment_mapper_circuit_targets.proof,
        &commitment_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .commitment_mapper_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &commitment_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .commitment_mapper_circuit_targets
            .verifier_circuit_target
            .circuit_digest,
        commitment_data.verifier_only.circuit_digest,
    );

    let proof = circuit_data.prove(pw)?;

    save_final_proof(&mut con, &proof).await?;

    println!(
        "{}",
        format!(
            "Proof size: {}",
            proof.to_bytes().len().to_string().magenta()
        )
        .blue()
        .bold()
    );

    println!("{}", "Final proof saved!".blue().bold());

    Ok(())
}
