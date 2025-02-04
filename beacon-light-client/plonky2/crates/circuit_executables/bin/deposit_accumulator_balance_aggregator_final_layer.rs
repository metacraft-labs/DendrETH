use circuit::{Circuit, CircuitDataType, CircuitInput, CircuitTargetType, SetWitness};
use circuit_executables::{
    cached_circuit_build::{build_circuit_cached, load_circuit_data_recursive},
    crud::{
        common::{
            fetch_deposit_accumulator_final_layer_input, fetch_proof, fetch_proof_balances,
            fetch_pubkey_commitment_mapper_proof_data, save_deposit_accumulator_final_proof,
        },
        proof_storage::{
            get_storage_key_from_matches, get_storages_from_matches, load_storage_config,
            proof_storage_definition_from_config, redis_connection_from_definition,
            MetadataBlobStorage,
        },
    },
    utils::CommandLineOptionsBuilder,
    wrap_final_layer_in_poseidon_bn128::wrap_final_layer_in_poseidon_bn_128,
};
use circuits::{
    deposit_accumulator_balance_aggregator_diva::{
        final_layer::{DependenciesCircuitData, DepositAccumulatorBalanceAggregatorDivaFinalLayer},
        first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
        inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
    },
    pubkey_commitment_mapper::inner_level::PubkeyCommitmentMapperIL,
    redis_storage_types::{
        DepositAccumulatorBalanceAggregatorDivaProofData, ValidatorsCommitmentMapperProofData,
    },
    validators_commitment_mapper::inner_level::ValidatorsCommitmentMapperInnerLevel,
};
use colored::Colorize;
use futures::StreamExt;
use redis::RedisError;
use std::println;

use anyhow::{Context, Result};
use num_traits::ToPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};

type ThisCircuit = DepositAccumulatorBalanceAggregatorDivaFinalLayer;

const STORAGE_ARG_NAMES: [&str; 3] = [
    "validators-commitment-mapper",
    "pubkey-commitment-mapper",
    "balance-verification",
];

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("final_layer")
        .with_protocol_options()
        .with_serialized_circuits_dir()
        .with_proof_storage_config()
        .add_proof_storages(&STORAGE_ARG_NAMES)
        .get_matches();

    println!("Establishing storage connections");

    let storage_config_filepath = matches.get_one::<String>("proof_storage_cfg").unwrap();
    let storage_config = load_storage_config(storage_config_filepath)?;

    let [mut vcm_storage, mut pcm_storage, mut bv_storage] =
        get_storages_from_matches(&matches, &storage_config, &STORAGE_ARG_NAMES).await?;

    let bv_storage_key = get_storage_key_from_matches(&matches, Some("balance-verification"))?;
    let subcon_def = proof_storage_definition_from_config(&storage_config, bv_storage_key)?;
    let subcon = redis_connection_from_definition(&subcon_def.metadata_storage).await?;
    let mut subcon = subcon.into_pubsub();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();

    let protocol = matches.value_of("protocol").unwrap();

    let (circuit_target, circuit_data, dependencies_circuit_data) = setup(serialized_circuits_dir)?;

    println!("Subscribing to pubsub channel \"balance-verification.proof\"");
    subcon.subscribe("balance-verification.proof").await?;
    let mut msg_stream = subcon.on_message();

    while let Some(data) = msg_stream.next().await {
        let maybe_payload: Result<String, RedisError> = data.get_payload();
        if !received_prove_message(maybe_payload) {
            continue;
        }

        println!("Starting proving");

        let circuit_input =
            fetch_deposit_accumulator_final_layer_input(&mut bv_storage.metadata, protocol).await?;

        let slot = circuit_input
            .slot
            .to_u64()
            .context("slot is not a valid u64")?;

        let block_number = circuit_input
            .execution_block_number
            .to_u64()
            .context("block_number is not a valid u64")?;

        let dependency_proofs = fetch_dependency_proofs(
            &dependencies_circuit_data,
            &mut bv_storage,
            &mut vcm_storage,
            &mut pcm_storage,
            protocol,
            slot,
            block_number,
        )
        .await?;

        let balance_verification_pis =
            DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(
                &dependency_proofs
                    .deposit_accumulator_balance_aggregator_proof
                    .public_inputs,
            );

        let proof = prove(
            &circuit_target,
            &circuit_data,
            &circuit_input,
            &dependency_proofs,
        )?;

        save_deposit_accumulator_final_proof(
            &mut bv_storage.metadata,
            protocol,
            &proof,
            &circuit_input,
            &balance_verification_pis,
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

        wrap_final_layer_in_poseidon_bn_128(
            &mut bv_storage.metadata,
            false,
            &circuit_data,
            proof,
            protocol.to_string(),
        )
        .await?;

        println!("{}", "Wrapper finished!".blue().bold());
    }

    Ok(())
}

fn received_prove_message(maybe_payload: Result<String, RedisError>) -> bool {
    match maybe_payload {
        Ok(payload) => {
            println!("Received payload: \"{payload}\"");
            payload == "prove"
        }
        Err(err) => {
            eprintln!("Err: {err}");
            false
        }
    }
}

fn load_dependencies_circuit_data(
    serialized_circuits_dir: &str,
) -> Result<Box<DependenciesCircuitData>> {
    let deposit_accumulator_balance_aggregator_circuit_data =
        load_circuit_data_recursive::<DepositAccumulatorBalanceAggregatorDivaInnerLevel>(
            serialized_circuits_dir,
            "deposit_accumulator_balance_aggregator_diva",
            32,
        )?;

    let validators_commitment_mapper_root_circuit_data =
        load_circuit_data_recursive::<ValidatorsCommitmentMapperInnerLevel>(
            serialized_circuits_dir,
            "commitment_mapper",
            40,
        )?;

    let validators_commitment_mapper_65536_circuit_data =
        load_circuit_data_recursive::<ValidatorsCommitmentMapperInnerLevel>(
            serialized_circuits_dir,
            "commitment_mapper",
            24,
        )?;

    let pubkey_commitment_mapper_circuit_data =
        load_circuit_data_recursive::<PubkeyCommitmentMapperIL>(
            serialized_circuits_dir,
            "pubkey_commitment_mapper",
            32,
        )?;

    let dependencies_circuit_data = DependenciesCircuitData {
        deposit_accumulator_balance_aggregator_circuit_data,
        validators_commitment_mapper_root_circuit_data,
        validators_commitment_mapper_65536_circuit_data,
        pubkey_commitment_mapper_circuit_data,
    };

    Ok(Box::new(dependencies_circuit_data))
}

type Proof = ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>;

async fn fetch_validators_commitment_mapper_proof(
    circuit_data: &CircuitDataType<ValidatorsCommitmentMapperInnerLevel>,
    storage: &mut MetadataBlobStorage,
    slot: u64,
    gindex: u64,
) -> Result<Proof> {
    let proof_data: ValidatorsCommitmentMapperProofData =
        fetch_proof(&mut storage.metadata, gindex, slot).await?;

    let proof_bytes = storage.blob.get_proof(proof_data.proof_key).await?;

    let proof = Proof::from_bytes(proof_bytes, &circuit_data.common)?;

    Ok(proof)
}

async fn fetch_pubkey_commitment_mapper_proof(
    circuit_data: &CircuitDataType<PubkeyCommitmentMapperIL>,
    storage: &mut MetadataBlobStorage,
    block_number: u64,
    protocol: &str,
) -> Result<Proof> {
    let proof_data =
        fetch_pubkey_commitment_mapper_proof_data(&mut storage.metadata, protocol, block_number)
            .await?;

    let proof_bytes = storage.blob.get_proof(proof_data.proof_key).await?;

    let proof = Proof::from_bytes(proof_bytes, &circuit_data.common)?;

    Ok(proof)
}

async fn fetch_balance_verification_proof(
    circuit_data: &CircuitDataType<DepositAccumulatorBalanceAggregatorDivaInnerLevel>,
    storage: &mut MetadataBlobStorage,
    protocol: &str,
) -> Result<Proof> {
    let proof_data = fetch_proof_balances::<DepositAccumulatorBalanceAggregatorDivaProofData>(
        &mut storage.metadata,
        protocol,
        32,
        0,
    )
    .await?;

    let proof_bytes = storage.blob.get_proof(proof_data.proof_key).await?;

    let proof = Proof::from_bytes(proof_bytes, &circuit_data.common)?;

    Ok(proof)
}

fn set_proof_targets_in_witness(
    pw: &mut PartialWitness<GoldilocksField>,
    circuit_target: &CircuitTargetType<ThisCircuit>,
    dependency_proofs: &DependencyProofs,
) {
    pw.set_proof_with_pis_target(
        &circuit_target.balance_aggregation_proof,
        &dependency_proofs.deposit_accumulator_balance_aggregator_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.validators_commitment_mapper_root_proof,
        &dependency_proofs.validators_commitment_mapper_root_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.validators_commitment_mapper_65536gindex_proof,
        &dependency_proofs.validators_commitment_mapper_65536_proof,
    );

    pw.set_proof_with_pis_target(
        &circuit_target.pubkey_commitment_mapper_proof,
        &dependency_proofs.pubkey_commitment_mapper_proof,
    );
}

#[allow(warnings)]
struct DependencyProofs {
    pub deposit_accumulator_balance_aggregator_proof: Proof,
    pub validators_commitment_mapper_root_proof: Proof,
    pub validators_commitment_mapper_65536_proof: Proof,
    pub pubkey_commitment_mapper_proof: Proof,
}

#[allow(warnings)]
async fn fetch_dependency_proofs(
    dependencies_circuit_data: &DependenciesCircuitData,
    bv_storage: &mut MetadataBlobStorage,
    vcm_storage: &mut MetadataBlobStorage,
    pcm_storage: &mut MetadataBlobStorage,
    protocol: &str,
    slot: u64,
    block_number: u64,
) -> Result<Box<DependencyProofs>> {
    let deposit_accumulator_balance_aggregator_proof = fetch_balance_verification_proof(
        &dependencies_circuit_data.deposit_accumulator_balance_aggregator_circuit_data,
        bv_storage,
        protocol,
    )
    .await?;

    let validators_commitment_mapper_root_proof = fetch_validators_commitment_mapper_proof(
        &dependencies_circuit_data.validators_commitment_mapper_root_circuit_data,
        vcm_storage,
        slot,
        1,
    )
    .await?;

    let validators_commitment_mapper_65536_proof = fetch_validators_commitment_mapper_proof(
        &dependencies_circuit_data.validators_commitment_mapper_65536_circuit_data,
        vcm_storage,
        slot,
        65536,
    )
    .await?;

    let pubkey_commitment_mapper_proof = fetch_pubkey_commitment_mapper_proof(
        &dependencies_circuit_data.pubkey_commitment_mapper_circuit_data,
        pcm_storage,
        block_number,
        protocol,
    )
    .await?;

    let dependency_proofs = DependencyProofs {
        deposit_accumulator_balance_aggregator_proof,
        validators_commitment_mapper_root_proof,
        validators_commitment_mapper_65536_proof,
        pubkey_commitment_mapper_proof,
    };

    Ok(Box::new(dependency_proofs))
}

fn prove(
    // circuit: &CircuitTargetAndData<DepositAccumulatorBalanceAggregatorDivaFinalLayer>,
    circuit_target: &CircuitTargetType<ThisCircuit>,
    circuit_data: &CircuitDataType<ThisCircuit>,
    circuit_input: &CircuitInput<ThisCircuit>,
    dependency_proofs: &DependencyProofs,
) -> Result<Proof> {
    let mut pw = PartialWitness::new();

    circuit_target.set_witness(&mut pw, circuit_input);
    set_proof_targets_in_witness(&mut pw, circuit_target, dependency_proofs);

    let proof = circuit_data.prove(pw)?;

    Ok(proof)
}

fn setup(
    serialized_circuits_dir: &str,
) -> Result<(
    CircuitTargetType<ThisCircuit>,
    CircuitDataType<ThisCircuit>,
    Box<DependenciesCircuitData>,
)> {
    println!("Loading dependencies circuit data");
    let dependencies_circuit_data = load_dependencies_circuit_data(serialized_circuits_dir)?;

    let (circuit_target, circuit_data) = build_circuit_cached(
        serialized_circuits_dir,
        "deposit_accumulator_final_layer",
        &|| ThisCircuit::build(&dependencies_circuit_data),
    );

    Ok((circuit_target, circuit_data, dependencies_circuit_data))
}

// Once
//  read circuit data of dependency circuits and keep it around for verification
//  build final layer circuit once and keep in memory
// Repeating
//   fetch final layer circuit input
//   fetch dependency proofs using the circuit input and circuit data
//   set partial witness using circuit input and the fetched proofs
//   generate the proof
//   save the outputs in redis
//   wrap the proof in poseidon bn 128
