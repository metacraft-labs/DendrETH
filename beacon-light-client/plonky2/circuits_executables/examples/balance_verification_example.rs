use std::{marker::PhantomData, println, time::Instant};

use anyhow::Result;
use circuits::{
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    targets_serialization::WriteTargets,
};
use circuits_executables::{
    crud::common::{fetch_validator_balance_input, write_to_file},
    provers::SetPWValues,
};
use futures_lite::future;
use plonky2::{iop::witness::PartialWitness, plonk::config::PoseidonGoldilocksConfig};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}
async fn async_main() -> Result<()> {
    let start = Instant::now();
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!("Redis connection took: {:?}", elapsed);

    let start = Instant::now();
    let (validators_balance_verification_targets, data) = build_validator_balance_circuit(8);
    let elapsed = start.elapsed();

    println!("Circuit generation took: {:?}", elapsed);

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_bytes = data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    write_to_file("validator_balance_circuit", &circuit_bytes).unwrap();

    let validator_balance_verification_targets_bytes = validators_balance_verification_targets
        .write_targets()
        .unwrap();

    write_to_file("targets", &validator_balance_verification_targets_bytes).unwrap();

    println!("Circuit size: {}", circuit_bytes.len());

    let mut pw = PartialWitness::new();

    let start = Instant::now();
    let validator_balance_input = fetch_validator_balance_input(&mut con, 0).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();

    validators_balance_verification_targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = data.prove(pw)?;

    // save_balance_proof();")

    println!("Public inputs: {:?}", proof.public_inputs);

    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);

    Ok(())
}
