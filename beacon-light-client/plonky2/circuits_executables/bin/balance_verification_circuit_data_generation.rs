use std::{fs, marker::PhantomData, println, time::Instant};

use anyhow::Result;
use circuits::{
    build_balance_inner_level_circuit::build_balance_inner_circuit,
    build_validator_balance_circuit::build_validator_balance_circuit,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
};

use futures_lite::future;

use jemallocator::Jemalloc;
use plonky2::{plonk::config::PoseidonGoldilocksConfig, util::serialization::Write};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let start = Instant::now();
    let (validators_balance_verification_targets, first_level_data) =
        build_validator_balance_circuit(8);
    let elapsed = start.elapsed();

    println!("Circuit generation took: {:?}", elapsed);

    let gate_serializer = DendrETHGateSerializer;

    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_bytes = first_level_data
        .to_bytes(&gate_serializer, &generator_serializer)
        .unwrap();

    write_to_file(&format!("{}.plonky2_circuit", 0), &circuit_bytes).unwrap();

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

    write_to_file(
        &format!("{}.plonky2_targets", 0),
        &validator_balance_verification_targets_bytes,
    )
    .unwrap();

    println!("Circuit size: {}", circuit_bytes.len());

    let mut prev_circuit_data = first_level_data;

    for i in 1..39 {
        let start = Instant::now();
        let (targets, data) = build_balance_inner_circuit(&prev_circuit_data);
        let elapsed = start.elapsed();

        println!("Circuit generation took: {:?}", elapsed);

        let circuit_bytes = data
            .to_bytes(&gate_serializer, &generator_serializer)
            .unwrap();

        write_to_file(&format!("{}.plonky2_circuit", i), &circuit_bytes).unwrap();

        let mut inner_level_targets = Vec::<u8>::new();

        inner_level_targets
            .write_target_proof_with_public_inputs(&targets.proof1)
            .unwrap();
        inner_level_targets
            .write_target_proof_with_public_inputs(&targets.proof2)
            .unwrap();
        inner_level_targets
            .write_target_verifier_circuit(&targets.verifier_circuit_target)
            .unwrap();
        inner_level_targets
            .write_target_bool(targets.is_zero)
            .unwrap();

        write_to_file(&format!("{}.plonky2_targets", i), &inner_level_targets).unwrap();

        println!("Circuit size: {}", circuit_bytes.len());
        prev_circuit_data = data;
    }

    Ok(())
}
