use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::Result;
use circuits::{
    build_final_circuit::build_final_circuit,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
};
use circuits_executables::{
    crud::{
        fetch_final_layer_input, fetch_proof, load_circuit_data, save_final_proof, BalanceProof,
        ValidatorProof,
    },
    provers::SetPWValues,
};
use clap::{App, Arg};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

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
        .arg(
            Arg::with_name("folder_name")
                .long("folder-name")
                .value_name("folder_name")
                .help("Sets the name of the folder proofs will be stored in")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("azure_account")
                .long("azure-account-name")
                .value_name("azure_account")
                .help("Sets the name of the azure account")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("azure_container")
                .long("azure-container-name")
                .value_name("azure_container")
                .help("Sets the name of the azure container")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("aws_endpoint_url")
                .long("aws-endpoint-url")
                .value_name("aws_endpoint_url")
                .help("Sets the aws endpoint url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("aws_region")
                .long("aws-region")
                .value_name("aws_region")
                .help("Sets the aws region")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("aws_bucket_name")
                .long("aws-bucket-name")
                .value_name("aws_bucket_name")
                .help("Sets the aws bucket name")
                .takes_value(true),
        )
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!("Redis connection took: {:?}", elapsed);

    let balance_data = load_circuit_data("37").unwrap();
    let commitment_data = load_circuit_data("commitment_mapper_40").unwrap();

    let (circuit_targets, circuit_data) = build_final_circuit(&balance_data, &commitment_data);

    let final_input_data = fetch_final_layer_input(&mut con).await?;

    let mut pw: PartialWitness<GoldilocksField> = PartialWitness::new();

    circuit_targets.set_pw_values(&mut pw, &final_input_data);

    let balance_proof: BalanceProof = fetch_proof(&mut con, 37, 0).await?;

    let balance_proof_bytes = balance_proof.proof;

    let balance_final_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_proof_bytes,
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

    let commitment_proof: ValidatorProof = fetch_proof(&mut con, 40, 0).await?;

    let commitment_proof_bytes = commitment_proof.proof;

    let commitment_final_proof = ProofWithPublicInputs::<
        GoldilocksField,
        PoseidonGoldilocksConfig,
        2,
    >::from_bytes(commitment_proof_bytes, &commitment_data.common)?;

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

    save_final_proof(
        &mut con,
        &proof,
        final_input_data
            .state_root
            .iter()
            .map(|x| *x as u64)
            .collect::<Vec<u64>>(),
        balance_proof.withdrawal_credentials,
        balance_proof.range_total_value,
        balance_proof.number_of_non_activated_validators,
        balance_proof.number_of_active_validators,
        balance_proof.number_of_exited_validators,
    )
    .await?;

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_data_bytes = circuit_data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    fs::write("final_layer.plonky2_circuit", circuit_data_bytes).unwrap();

    println!("Final proof saved!");

    Ok(())
}
