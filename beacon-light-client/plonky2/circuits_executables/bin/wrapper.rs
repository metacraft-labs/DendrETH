use std::{fs, println, time::Instant};

use anyhow::Result;
use circuits_executables::{
    crud::{load_circuit_data, FinalProof},
    poseidon_bn128_config::PoseidonBN128GoldilocksConfig,
    validator_commitment_constants::get_validator_commitment_constants,
};
use clap::{App, Arg};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::AsyncCommands;

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

    println!("Redis connection took: {:?}", elapsed);

    let final_layer_circuit = load_circuit_data("final_layer").unwrap();

    let standard_recursion_config = CircuitConfig::standard_recursion_config();

    let config = CircuitConfig {
        fri_config: FriConfig {
            rate_bits: 6,
            cap_height: 4,
            proof_of_work_bits: 16,
            reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
            num_query_rounds: 14,
        },
        ..standard_recursion_config
    };

    let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(config);

    let verifier_target = builder.constant_verifier_data(&final_layer_circuit.verifier_only);

    let proof_target = builder.add_virtual_proof_with_pis(&final_layer_circuit.common);

    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &proof_target.clone(),
        &verifier_target,
        &final_layer_circuit.common,
    );

    builder.register_public_inputs(&proof_target.public_inputs);

    let proof_str: String = con
        .get(get_validator_commitment_constants().final_layer_proof_key)
        .await?;

    let final_layer_proof: FinalProof = serde_json::from_str(&proof_str)?;

    println!("Final Layer Proof: {:?}", final_layer_proof.balance_sum);

    let final_layer_proof = final_layer_proof.proof;

    let final_proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> =
        ProofWithPublicInputs::from_bytes(final_layer_proof, &final_layer_circuit.common)?;

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&proof_target, &final_proof);

    let circuit_data = builder.build::<PoseidonBN128GoldilocksConfig>();

    let proof = circuit_data.prove(pw)?;

    let proof_json = serde_json::to_string(&proof).unwrap();

    fs::write("proof_with_public_inputs.json", proof_json).unwrap();

    let common_circuit_data = circuit_data.common;
    let common_circuit_data = serde_json::to_string(&common_circuit_data).unwrap();

    fs::write("common_circuit_data.json", common_circuit_data).unwrap();

    let verifier_only_circuit_data = serde_json::to_string(&circuit_data.verifier_only).unwrap();

    fs::write(
        "verifier_only_circuit_data.json",
        verifier_only_circuit_data,
    )
    .unwrap();

    Ok(())
}
