use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::{Ok, Result};
use circuits::{
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{self, DendrETHGateSerializer, DendrETHGeneratorSerializer},
    validator_hash_tree_root_poseidon::ValidatorPoseidon,
};
use circuits_executables::{
    crud::{fetch_validator_balance_input, read_from_file},
    provers::{set_boolean_pw_values, set_pw_values, set_validator_pw_values},
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::{BoolTarget, Target},
        witness::PartialWitness,
    },
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::{Buffer, Read, Write},
};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let start = Instant::now();

    let mut balances = Vec::<Vec<BoolTarget>>::new();
    let mut validators = Vec::<ValidatorPoseidon>::new();

    let target_bytes = read_from_file("targets")?;
    let mut target_buffer = Buffer::new(&target_bytes);

    for _ in 0..2 {
        balances.push(target_buffer.read_target_bool_vec().unwrap());
    }

    for _ in 0..8 {
        validators.push(ValidatorPoseidon {
            pubkey: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            withdrawal_credentials: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            effective_balance: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            slashed: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            activation_eligibility_epoch: target_buffer
                .read_target_vec()
                .unwrap()
                .try_into()
                .unwrap(),
            activation_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            exit_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            withdrawable_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
        });
    }

    let withdrawal_credentials = target_buffer.read_target_vec().unwrap();

    let circuit_data_bytes = read_from_file("validator_balance_circuit")?;

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let data = CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        &circuit_data_bytes,
        &gate_serializer,
        &generator_serializer,
    ).unwrap();


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

    for i in 0..validator_balance_input.balances.len() {
        set_boolean_pw_values(
            &mut pw,
            &balances[i],
            validator_balance_input.balances[i].clone(),
        );
    }

    for i in 0..validator_balance_input.validators.len() {
        set_validator_pw_values(
            &mut pw,
            &validators[i],
            &validator_balance_input.validators[i],
        );
    }

    set_pw_values(
        &mut pw,
        &withdrawal_credentials,
        validator_balance_input.withdrawal_credentials,
    );

    let proof = data.prove(pw)?;


    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);
    println!("Public inputs: {:?}", proof.public_inputs);

    Ok(())
}
