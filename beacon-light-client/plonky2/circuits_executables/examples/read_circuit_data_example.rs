use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::{Ok, Result};
use circuits::{
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    targets_serialization::ReadTargets,
    validator_balance_circuit::ValidatorBalanceVerificationTargets,
    validator_commitment_mapper::ValidatorCommitmentTargets,
};
use circuits_executables::{
    crud::{fetch_validator, fetch_validator_balance_input, read_from_file},
    poseidon_bn128_config::PoseidonBN128GoldilocksConfig,
    provers::SetPWValues,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    fri::{reduction_strategies::FriReductionStrategy, FriConfig},
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
    },
    util::serialization::Buffer,
};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let start = Instant::now();

    let target_bytes = read_from_file("commitment_mapper_0.plonky2_targets")?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let validator_targets = ValidatorCommitmentTargets::read_targets(&mut target_buffer).unwrap();

    let circuit_data_bytes = read_from_file("commitment_mapper_0.plonky2_circuit")?;

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let data = CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        &circuit_data_bytes,
        &gate_serializer,
        &generator_serializer,
    )
    .unwrap();

    let elapsed = start.elapsed();

    println!("Loading circuit took {:?}", elapsed);

    let start = Instant::now();
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!("Redis connection took: {:?}", elapsed);

    let start = Instant::now();
    let validator_balance_input = fetch_validator(&mut con, 0).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();
    let mut pw = PartialWitness::new();

    validator_targets
        .validator
        .set_pw_values(&mut pw, &validator_balance_input);

    let proof = data.prove(pw)?;

    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);
    println!("Public inputs: {:?}", proof.public_inputs);

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

    let verifier_target = builder.constant_verifier_data(&data.verifier_only);

    let proof_target = builder.add_virtual_proof_with_pis(&data.common);

    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &proof_target.clone(),
        &verifier_target,
        &data.common,
    );

    let public_inputs_inner_proof = proof_target.public_inputs.clone();

    builder.register_public_inputs(&public_inputs_inner_proof);

    println!("Starting building final proof");
    let start = Instant::now();

    let data = builder.build::<PoseidonBN128GoldilocksConfig>();

    let elapsed = start.elapsed();

    println!("Building took: {:?}", elapsed);

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&proof_target, &proof);

    println!("Starting proofing final proof");

    let start = Instant::now();

    let proof = data.prove(pw).unwrap();

    let elapsed = start.elapsed();

    println!("Proving took: {:?}", elapsed);

    let json = serde_json::to_string(&proof).unwrap();

    fs::write("proof_with_public_inputs.json", json).unwrap();

    let common_circuit_data = data.common;
    let common_circuit_data = serde_json::to_string(&common_circuit_data).unwrap();

    fs::write("common_circuit_data.json", common_circuit_data).unwrap();

    let verifier_only_circuit_data = serde_json::to_string(&data.verifier_only).unwrap();

    fs::write(
        "verifier_only_circuit_data.json",
        verifier_only_circuit_data,
    )
    .unwrap();

    Ok(())
}
