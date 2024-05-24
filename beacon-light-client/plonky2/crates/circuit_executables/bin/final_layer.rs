#![feature(generic_const_exprs)]

use circuit::{Circuit, SetWitness};
use circuit_executables::{
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::{
        common::{
            fetch_final_layer_input, fetch_proof, fetch_proof_balances, load_circuit_data,
            save_final_proof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    utils::{parse_config_file, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    final_layer::BalanceVerificationFinalCircuit,
    types::{BalanceProof, ValidatorProof},
    utils::utils::bits_to_bytes,
};
use colored::Colorize;
use itertools::Itertools;
use std::{println, time::Instant};

use anyhow::Result;
use futures_lite::future;
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("final_layer")
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
        .with_proof_storage_options()
        .with_protocol_options()
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let mut proof_storage = create_proof_storage(&matches).await;

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!(
        "{}",
        format!("Redis connection took: {:?}", elapsed).yellow()
    );

    let verification_circuit_data = (
        load_circuit_data(&format!(
            "{SERIALIZED_CIRCUITS_DIR}/balance_verification_37",
        ))
        .unwrap(),
        load_circuit_data(&format!("{SERIALIZED_CIRCUITS_DIR}/commitment_mapper_40")).unwrap(),
    );

    let (circuit_targets, circuit_data) =
        BalanceVerificationFinalCircuit::<1>::build(&verification_circuit_data);

    let (balance_data, commitment_data) = verification_circuit_data;

    let protocol = matches.value_of("protocol").unwrap();

    let final_input_data = fetch_final_layer_input(&mut con, protocol).await?;

    let mut pw: PartialWitness<GoldilocksField> = PartialWitness::new();
    circuit_targets.set_witness(&mut pw, &final_input_data);

    // TODO: don't hard code the generics
    let balance_proof: BalanceProof<8, 1> = fetch_proof_balances(&mut con, protocol, 37, 0).await?;
    let balance_proof_bytes = proof_storage.get_proof(balance_proof.proof_key).await?;

    let balance_final_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_proof_bytes,
            &balance_data.common,
        )?;

    pw.set_proof_with_pis_target(
        &circuit_targets.balance_verification_proof_target.proof,
        &balance_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .balance_verification_proof_target
            .verifier_circuit_target
            .constants_sigmas_cap,
        &balance_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .balance_verification_proof_target
            .verifier_circuit_target
            .circuit_digest,
        balance_data.verifier_only.circuit_digest,
    );

    let commitment_proof: ValidatorProof =
        fetch_proof(&mut con, 1, final_input_data.slot.to_u64().unwrap()).await?;

    let commitment_proof_bytes = proof_storage.get_proof(commitment_proof.proof_key).await?;

    let commitment_final_proof = ProofWithPublicInputs::<
        GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >::from_bytes(commitment_proof_bytes, &commitment_data.common)?;

    pw.set_proof_with_pis_target(
        &circuit_targets.commitment_mapper_proof_target.proof,
        &commitment_final_proof,
    );

    pw.set_cap_target(
        &circuit_targets
            .commitment_mapper_proof_target
            .verifier_circuit_target
            .constants_sigmas_cap,
        &commitment_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        circuit_targets
            .commitment_mapper_proof_target
            .verifier_circuit_target
            .circuit_digest,
        commitment_data.verifier_only.circuit_digest,
    );

    let proof = circuit_data.prove(pw)?;

    let withdrawal_credentials = balance_proof.public_inputs.withdrawal_credentials.to_vec();
    let withdrawal_credentials = withdrawal_credentials
        .iter()
        .map(|credentials| hex::encode(bits_to_bytes(credentials.as_slice())))
        .collect_vec();

    // TODO: read the balance verification proof public inputs
    save_final_proof(
        &mut con,
        protocol.to_string(),
        &proof,
        hex::encode(bits_to_bytes(final_input_data.block_root.as_slice())),
        // TODO: read these off the public inputs, not the redis data
        withdrawal_credentials,
        balance_proof
            .public_inputs
            .range_total_value
            .to_u64()
            .unwrap(),
        balance_proof
            .public_inputs
            .number_of_non_activated_validators,
        balance_proof.public_inputs.number_of_active_validators,
        balance_proof
            .public_inputs
            .number_of_non_activated_validators,
        balance_proof.public_inputs.number_of_slashed_validators,
    )
    .await?;

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

    println!("{}", "Running wrapper...".blue().bold());

    wrap_final_layer_in_poseidon_bn_128(con, false, circuit_data, proof, protocol.to_string())
        .await?;

    println!("{}", "Wrapper finished!".blue().bold());

    Ok(())
}
