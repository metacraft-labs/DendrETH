use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::Result;
use circuits::{
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{self, DendrETHGateSerializer, DendrETHGeneratorSerializer},
};
use circuits_executables::{
    crud::{fetch_validator_balance_input, write_to_file},
    provers::{set_boolean_pw_values, set_pw_values, set_validator_pw_values},
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Write,
};

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

    let mut validator_balance_verification_targets_bytes = Vec::<u8>::new();

    for i in 0..validators_balance_verification_targets.balances.len() {
        validator_balance_verification_targets_bytes
            .write_target_bool_vec(&validators_balance_verification_targets.balances[i])
            .unwrap();
    }

    for i in 0..validators_balance_verification_targets.validators.len() {
        validator_balance_verification_targets_bytes
            .write_target_vec(&validators_balance_verification_targets.validators[i].pubkey)
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(
                &validators_balance_verification_targets.validators[i].withdrawal_credentials,
            )
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(
                &validators_balance_verification_targets.validators[i].effective_balance,
            )
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(&validators_balance_verification_targets.validators[i].slashed)
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(
                &validators_balance_verification_targets.validators[i].activation_eligibility_epoch,
            )
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(
                &validators_balance_verification_targets.validators[i].activation_epoch,
            )
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(&validators_balance_verification_targets.validators[i].exit_epoch)
            .unwrap();
        validator_balance_verification_targets_bytes
            .write_target_vec(
                &validators_balance_verification_targets.validators[i].withdrawable_epoch,
            )
            .unwrap();
    }

    validator_balance_verification_targets_bytes
        .write_target_vec(&validators_balance_verification_targets.withdrawal_credentials)
        .unwrap();

    write_to_file("targets", &validator_balance_verification_targets_bytes).unwrap();

    println!("Circuit size: {}", circuit_bytes.len());

    let mut pw = PartialWitness::new();

    let start = Instant::now();
    let validator_balance_input = fetch_validator_balance_input(&mut con, 0).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();

    for i in 0..validator_balance_input.balances.len() {
        set_boolean_pw_values(
            &mut pw,
            &validators_balance_verification_targets.balances[i],
            validator_balance_input.balances[i].clone(),
        );
    }

    for i in 0..validator_balance_input.validators.len() {
        set_validator_pw_values(
            &mut pw,
            &validators_balance_verification_targets.validators[i],
            &validator_balance_input.validators[i],
        );
    }

    set_pw_values(
        &mut pw,
        &validators_balance_verification_targets.withdrawal_credentials,
        validator_balance_input.withdrawal_credentials,
    );

    let proof = data.prove(pw)?;

    // save_balance_proof();")

    println!("Public inputs: {:?}", proof.public_inputs);

    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);

    Ok(())
}
