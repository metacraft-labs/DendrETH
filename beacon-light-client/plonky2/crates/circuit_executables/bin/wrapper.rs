#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::{println, time::Instant};

use anyhow::Result;
use circuit::Circuit;
use circuit_executables::{
    crud::common::load_circuit_data,
    db_constants::DB_CONSTANTS,
    utils::{get_default_config, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    redis_storage_types::BalanceVerificationFinalProofData,
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::final_layer::BalanceVerificationFinalCircuit,
};
use clap::Arg;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> Result<()> {
    let common_config = get_default_config().unwrap();

    let matches = CommandLineOptionsBuilder::new("wrapper")
        .with_redis_options(
            &common_config.redis_host,
            common_config.redis_port,
            &common_config.redis_auth,
        )
        .with_protocol_options()
        .with_serialized_circuits_dir()
        .arg(
            Arg::with_name("compile")
                .short('c')
                .long("compile")
                .help("Compile the circuit")
                .takes_value(false),
        )
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let redis_connection = matches.value_of("redis_connection").unwrap();
    let compile_circuit = matches.is_present("compile");

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();
    println!("Redis connection took: {:?}", elapsed);
    let verification_circuit_data = (
        load_circuit_data::<BalanceVerificationFinalCircuit<1>>(
            serialized_circuits_dir,
            "balance_verification_37",
        )?,
        load_circuit_data::<ValidatorsCommitmentMapperFirstLevel>(
            serialized_circuits_dir,
            "commitment_mapper_40",
        )?,
    );

    let (_, circuit_data) = BalanceVerificationFinalCircuit::<1>::build(&verification_circuit_data);

    let protocol = matches.value_of("protocol").unwrap();

    let proof_str: String = con
        .get(format!(
            "{}:{}",
            protocol.to_string(),
            DB_CONSTANTS.final_layer_proof_key
        ))
        .await?;
    let final_layer_proof: BalanceVerificationFinalProofData = serde_json::from_str(&proof_str)?;
    let final_layer_proof = final_layer_proof.proof;
    let final_layer_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> =
        ProofWithPublicInputs::from_bytes(final_layer_proof, &circuit_data.common)?;

    wrap_final_layer_in_poseidon_bn_128(
        &mut con,
        compile_circuit,
        circuit_data,
        final_layer_proof,
        protocol.to_string(),
    )
    .await?;

    Ok(())
}
