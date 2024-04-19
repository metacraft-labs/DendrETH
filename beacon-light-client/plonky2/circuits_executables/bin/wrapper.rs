use std::{println, time::Instant};

use anyhow::Result;
use circuits_executables::{
    crud::common::{load_circuit_data, FinalProof},
    utils::{parse_config_file, CommandLineOptionsBuilder},
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
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
    let common_config = parse_config_file("../common_config.json".to_owned()).unwrap();

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
    let final_layer_circuit: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2> =
        load_circuit_data("circuits/final_layer").unwrap();

    let protocol = matches.value_of("protocol").unwrap();

    let proof_str: String = con
        .get(format!(
            "{}:{}",
            protocol.to_string(),
            VALIDATOR_COMMITMENT_CONSTANTS.final_layer_proof_key
        ))
        .await?;
    let final_layer_proof: FinalProof = serde_json::from_str(&proof_str)?;
    let final_layer_proof = final_layer_proof.proof;
    let final_layer_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> =
        ProofWithPublicInputs::from_bytes(final_layer_proof, &final_layer_circuit.common)?;

    wrap_final_layer_in_poseidon_bn_128(
        con,
        compile_circuit,
        final_layer_circuit,
        final_layer_proof,
        protocol.to_string(),
    )
    .await?;

    Ok(())
}
