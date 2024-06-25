use circuit::{Circuit, SetWitness};
use circuit_executables::{
    cached_circuit_build::{build_circuit_cached, SERIALIZED_CIRCUITS_DIR},
    crud::{
        common::{
            fetch_deposit_accumulator_final_layer_input, fetch_proof, fetch_proof_balances,
            fetch_pubkey_commitment_mapper_proof, load_circuit_data,
            save_deposit_accumulator_final_proof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    utils::{parse_config_file, CommandLineOptionsBuilder},
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    deposit_accumulator_balance_aggregator_diva::{
        final_layer::DepositAccumulatorBalanceAggregatorDivaFinalLayer,
        first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    },
    redis_storage_types::{
        DepositAccumulatorBalanceAggregatorDivaProofData, ValidatorsCommitmentMapperProofData,
    },
    utils::bits_to_bytes,
};
use colored::Colorize;
use std::{println, time::Instant};

use anyhow::Result;
use futures_lite::future;
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("final_layer")
        .with_redis_options(&common_config.redis_host, common_config.redis_port, &common_config.redis_auth)
        .with_proof_storage_options()
        .with_protocol_options()
        .get_matches();

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

    let circuit_input = fetch_deposit_accumulator_final_layer_input(&mut con, protocol).await?;

    let balance_proof_data =
        fetch_proof_balances::<DepositAccumulatorBalanceAggregatorDivaProofData>(
            &mut con, protocol, 32, 0,
        )
        .await?;

    let balance_verification_proof_bytes = proof_storage
        .get_proof(balance_proof_data.proof_key)
        .await?;

    let balance_verification_circuit_data = load_circuit_data(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/deposit_accumulator_balance_aggregator_diva_32",
    ))
    .unwrap();

    let balance_verification_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            balance_verification_proof_bytes,
            &balance_verification_circuit_data.common,
        )?;

    let validators_commitment_mapper_root_proof_data: ValidatorsCommitmentMapperProofData =
        fetch_proof(&mut con, 1, circuit_input.slot.to_u64().unwrap()).await?;

    let validators_commitment_mapper_root_proof_bytes = proof_storage
        .get_proof(validators_commitment_mapper_root_proof_data.proof_key)
        .await?;

    let validators_commitment_mapper_65536_gindex_proof_data: ValidatorsCommitmentMapperProofData =
        fetch_proof(&mut con, 65536, circuit_input.slot.to_u64().unwrap()).await?;

    let validators_commitment_mapper_65536gindex_proof_bytes = proof_storage
        .get_proof(validators_commitment_mapper_65536_gindex_proof_data.proof_key)
        .await?;

    let validators_commitment_mapper_root_circuit_data =
        load_circuit_data(&format!("{SERIALIZED_CIRCUITS_DIR}/commitment_mapper_40")).unwrap();

    let validators_commitment_mapper_65536gindex_circuit_data =
        load_circuit_data(&format!("{SERIALIZED_CIRCUITS_DIR}/commitment_mapper_24")).unwrap();

    let validators_commitment_root_mapper_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            validators_commitment_mapper_root_proof_bytes,
            &validators_commitment_mapper_root_circuit_data.common,
        )?;

    let validators_commitment_mapper_65536gindex_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            validators_commitment_mapper_65536gindex_proof_bytes,
            &validators_commitment_mapper_65536gindex_circuit_data.common,
        )?;

    let block_number = circuit_input.execution_block_number.to_u64().unwrap();
    let pubkey_commitment_mapper_proof =
        fetch_pubkey_commitment_mapper_proof(&mut con, protocol, block_number).await?;

    let pubkey_commitment_mapper_proof_bytes = proof_storage
        .get_proof(pubkey_commitment_mapper_proof.proof_key)
        .await?;

    let pubkey_commitment_mapper_circuit_data = load_circuit_data(&format!(
        "{SERIALIZED_CIRCUITS_DIR}/pubkey_commitment_mapper_32"
    ))
    .unwrap();

    let pubkey_commitment_mapper_proof =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            pubkey_commitment_mapper_proof_bytes,
            &pubkey_commitment_mapper_circuit_data.common,
        )?;

    let verification_circuit_data = (
        balance_verification_circuit_data,
        validators_commitment_mapper_root_circuit_data,
        validators_commitment_mapper_65536gindex_circuit_data,
        pubkey_commitment_mapper_circuit_data,
    );

    let (circuit_target, circuit_data) =
        build_circuit_cached("deposit_accumulator_final_layer", &|| {
            DepositAccumulatorBalanceAggregatorDivaFinalLayer::build(&verification_circuit_data)
        });

    let mut pw = PartialWitness::new();

    circuit_target.set_witness(&mut pw, &circuit_input);

    pw.set_proof_with_pis_target(
        &circuit_target.balance_aggregation_proof,
        &balance_verification_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.validators_commitment_mapper_root_proof,
        &validators_commitment_root_mapper_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.validators_commitment_mapper_65536gindex_proof,
        &validators_commitment_mapper_65536gindex_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.pubkey_commitment_mapper_proof,
        &pubkey_commitment_mapper_proof,
    );

    let proof = circuit_data.prove(pw)?;

    let balance_verification_pis =
        DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
            &balance_verification_proof.public_inputs,
        );

    save_deposit_accumulator_final_proof(
        &mut con,
        protocol.to_string(),
        &proof,
        circuit_input.slot.to_u64().unwrap(),
        circuit_input.execution_block_number.to_u64().unwrap(),
        hex::encode(bits_to_bytes(circuit_input.block_root.as_slice())),
        balance_verification_pis
            .accumulated_data
            .balance
            .to_u64()
            .unwrap(),
        balance_verification_pis
            .accumulated_data
            .validator_status_stats
            .non_activated_count,
        balance_verification_pis
            .accumulated_data
            .validator_status_stats
            .active_count,
        balance_verification_pis
            .accumulated_data
            .validator_status_stats
            .exited_count,
        balance_verification_pis
            .accumulated_data
            .validator_status_stats
            .slashed_count,
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
