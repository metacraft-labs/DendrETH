#![feature(generic_const_exprs)]
use std::{println, time::Instant};

use anyhow::Result;
use circuit::Circuit;
use circuit_executables::{
    constants::SERIALIZED_CIRCUITS_DIR,
    crud::common::load_circuit_data,
    db_constants::DB_CONSTANTS,
    utils::{parse_config_file, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{final_layer::BalanceVerificationFinalCircuit, types::FinalProof};
use clap::Arg;
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::AsyncCommands;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("wrapper")
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
        .with_protocol_options()
        .arg(
            Arg::with_name("compile")
                .short('c')
                .long("compile")
                .help("Compile the circuit")
                .takes_value(false),
        )
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();
    let compile_circuit = matches.is_present("compile");

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();
    println!("Redis connection took: {:?}", elapsed);
    let verification_circuit_data = (
        load_circuit_data(&format!(
            "{SERIALIZED_CIRCUITS_DIR}/balance_verification_37",
        ))
        .unwrap(),
        load_circuit_data(&format!("{SERIALIZED_CIRCUITS_DIR}/commitment_mapper_40")).unwrap(),
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
    let final_layer_proof: FinalProof = serde_json::from_str(&proof_str)?;
    let final_layer_proof = final_layer_proof.proof;
    let final_layer_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> =
        ProofWithPublicInputs::from_bytes(final_layer_proof, &circuit_data.common)?;

    wrap_final_layer_in_poseidon_bn_128(
        con,
        compile_circuit,
        circuit_data,
        final_layer_proof,
        protocol.to_string(),
    )
    .await?;

    Ok(())
}
