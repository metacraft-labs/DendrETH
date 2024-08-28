#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::{Circuit, SetWitness};
use circuit_executables::{
    crud::{
        common::{
            fetch_final_layer_input, fetch_proof, fetch_proof_balances, load_circuit_data,
            save_final_proof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    utils::{get_default_config, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    redis_storage_types::{
        ValidatorsCommitmentMapperProofData, WithdrawalCredentialsBalanceVerificationProofData,
    },
    utils::bits_to_bytes,
    validators_commitment_mapper::inner_level::ValidatorsCommitmentMapperInnerLevel,
    withdrawal_credentials_balance_aggregator::{
        final_layer::BalanceVerificationFinalCircuit,
        first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
        inner_level::WithdrawalCredentialsBalanceAggregatorInnerLevel,
    },
};
use colored::Colorize;
use itertools::Itertools;
use std::{println, time::Instant};

use anyhow::Result;
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

const VALIDATORS_COUNT: usize = 8;
const WITHDRAWAL_CREDENTIALS_COUNT: usize = 1;

#[tokio::main]
async fn main() -> Result<()> {
    let common_config = get_default_config().unwrap();

    let matches = CommandLineOptionsBuilder::new("final_layer")
        .with_redis_options(
            &common_config.redis_host,
            common_config.redis_port,
            &common_config.redis_auth,
        )
        .with_proof_storage_options()
        .with_protocol_options()
        .with_serialized_circuits_dir()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let redis_connection = matches.value_of("redis_connection").unwrap();
    let protocol = matches.value_of("protocol").unwrap();

    let mut proof_storage = create_proof_storage(&matches).await;

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!(
        "{}",
        format!("Redis connection took: {:?}", elapsed).yellow()
    );

    let circuit_input = fetch_final_layer_input(&mut con, protocol).await?;

    let balance_proof_data: WithdrawalCredentialsBalanceVerificationProofData<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    > = fetch_proof_balances(&mut con, protocol, 37, 0).await?;

    let balance_verification_proof_bytes = proof_storage
        .get_proof(balance_proof_data.proof_key)
        .await?;

    let balance_verification_circuit_data = load_circuit_data::<
        WithdrawalCredentialsBalanceAggregatorInnerLevel<8, 1>,
    >(
        serialized_circuits_dir, "balance_verification_37"
    )
    .unwrap();

    let balance_verification_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_verification_proof_bytes,
            &balance_verification_circuit_data.common,
        )?;

    let validators_commitment_mapper_proof_data: ValidatorsCommitmentMapperProofData =
        fetch_proof(&mut con, 1, circuit_input.slot.to_u64().unwrap()).await?;

    let validators_commitment_mapper_proof_bytes = proof_storage
        .get_proof(validators_commitment_mapper_proof_data.proof_key)
        .await?;

    let validators_commitment_mapper_circuit_data =
        load_circuit_data::<ValidatorsCommitmentMapperInnerLevel>(
            serialized_circuits_dir,
            "commitment_mapper_40",
        )
        .unwrap();

    let validators_commitment_mapper_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            validators_commitment_mapper_proof_bytes,
            &validators_commitment_mapper_circuit_data.common,
        )?;

    let verification_circuit_data = (
        balance_verification_circuit_data,
        validators_commitment_mapper_circuit_data,
    );

    let (circuit_target, circuit_data) = BalanceVerificationFinalCircuit::<
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::build(&verification_circuit_data);

    let mut pw = PartialWitness::new();

    circuit_target.set_witness(&mut pw, &circuit_input);

    pw.set_proof_with_pis_target(
        &circuit_target.balance_verification_proof,
        &balance_verification_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.validators_commitment_mapper_proof,
        &validators_commitment_mapper_proof,
    );

    let proof = circuit_data.prove(pw)?;

    let balance_verification_pis =
        WithdrawalCredentialsBalanceAggregatorFirstLevel::<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >::read_public_inputs(&balance_verification_proof.public_inputs);

    let withdrawal_credentials = balance_verification_pis
        .withdrawal_credentials
        .to_vec()
        .iter()
        .map(|credentials| hex::encode(bits_to_bytes(credentials.as_slice())))
        .collect_vec();

    save_final_proof(
        &mut con,
        protocol.to_string(),
        &proof,
        hex::encode(bits_to_bytes(circuit_input.block_root.as_slice())),
        withdrawal_credentials,
        balance_verification_pis.range_total_value.to_u64().unwrap(),
        balance_verification_pis.number_of_non_activated_validators,
        balance_verification_pis.number_of_active_validators,
        balance_verification_pis.number_of_exited_validators,
        balance_verification_pis.number_of_slashed_validators,
    )
    .await?;

    println!(
        "{}",
        format!(
            "Proof size: {}",
            proof.to_bytes().len().to_string().magenta()
        )
        .blue()
        .bold()
    );

    println!("{}", "Final proof saved!".blue().bold());

    println!("{}", "Running wrapper...".blue().bold());

    wrap_final_layer_in_poseidon_bn_128(con, false, circuit_data, proof, protocol.to_string())
        .await?;

    println!("{}", "Wrapper finished!".blue().bold());

    Ok(())
}
