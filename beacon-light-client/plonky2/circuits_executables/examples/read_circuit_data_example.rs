use std::{marker::PhantomData, println, time::Instant};

use anyhow::{Ok, Result};
use circuits::{
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    targets_serialization::ReadTargets,
    validator_balance_circuit::ValidatorBalanceVerificationTargets,
};
use circuits_executables::{
    crud::common::{fetch_validator_balance_input, read_from_file},
    provers::SetPWValues,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
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

    let target_bytes = read_from_file("0.plonky2_targets")?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let validator_targets =
        ValidatorBalanceVerificationTargets::read_targets(&mut target_buffer).unwrap();

    let circuit_data_bytes = read_from_file("0.plonky2_circuit")?;

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
    let validator_balance_input = fetch_validator_balance_input(&mut con, 0).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();
    let mut pw = PartialWitness::new();

    validator_targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = data.prove(pw)?;

    println!("proof size {}", proof.to_bytes().len());

    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);
    println!("Public inputs: {:?}", proof.public_inputs);

    Ok(())
}
