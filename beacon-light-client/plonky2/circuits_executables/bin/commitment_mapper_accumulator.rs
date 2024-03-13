use anyhow::Result;
use colored::Colorize;
use num::BigUint;
use std::{marker::PhantomData, thread, time::Duration};

use circuits::{
    biguint::WitnessBigUint,
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    targets_serialization::ReadTargets,
    utils::SetBytesArray,
    validator_accumulator_commitment_mapper::ValidatorAccumulatorCommitmentTargets,
};

use circuits_executables::{
    commitment_mapper_task::CommitmentMapperAccumulatorTask,
    crud::{
        common::{
            fetch_accumulator_proofs, fetch_redis_json_object, fetch_zero_accumulator_proof,
            get_depth_for_gindex, load_circuit_data, read_from_file,
            save_validator_accumulator_proof, save_zero_validator_accumulator_proof, ProofProvider,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    provers::handle_commitment_mapper_inner_level_proof,
    utils::{
        format_hex, gindex_from_validator_index, parse_config_file, CommandLineOptionsBuilder,
    },
    validator::{ValidatorAccumulatorInput, VALIDATOR_REGISTRY_LIMIT},
use circuits_executables::crud::get_depth_for_gindex;
use circuits_executables::{
    commitment_mapper_task::CommitmentMapperAccumulatorTask,
    crud::{
        fetch_accumulator_proofs, fetch_validator_accumulator, fetch_zero_accumulator_proof,
        load_circuit_data, read_from_file, save_validator_accumulator_proof,
        save_zero_validator_accumulator_proof, ProofProvider,
    },
    provers::handle_commitment_mapper_inner_level_proof,
    utils::{format_hex, get_depth_for_gindex, gindex_from_validator_index},
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use futures_lite::future;
use jemallocator::Jemalloc;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "commitment_mapper_accumulator";

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../common_config.json".to_owned()).unwrap();
    let matches = CommandLineOptionsBuilder::new("commitment_mapper_accumulator")
        .with_redis_options(&common_config.redis_host, &common_config.redis_port)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let protocol = matches.value_of("protocol").unwrap();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;
    let mut proof_storage = create_proof_storage(&matches).await;

    let queue = WorkQueue::new(KeyPrefix::new(
        VALIDATOR_COMMITMENT_CONSTANTS
            .validator_accumulator_proof_queue
            .to_owned(),
    ));

    let first_level_circuit_data = load_circuit_data(&format!("{}_{}", CIRCUIT_NAME, 0))?;
    let validator_commitment = get_first_level_targets()?;

    let mut inner_circuits: Vec<(
        CommitmentMapperInnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::new();

    for i in 1..=32 {
        inner_circuits.push((get_inner_targets(i)?, {
            let file_name: &str = &format!("{}_{}", CIRCUIT_NAME, i);
            let gate_serializer = DendrETHGateSerializer;
            let generator_serializer = DendrETHGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            };

            let circuit_data_bytes = read_from_file(&format!("{}.plonky2_circuit", file_name))?;

            CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
                &circuit_data_bytes,
                &gate_serializer,
                &generator_serializer,
            )
            .unwrap()
        }));
    }

    let stop_after = matches
        .value_of("stop_after")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let lease_for = matches
        .value_of("lease_for")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    loop {
        println!("{}", "Waiting for task...".yellow());

        let Some(queue_item) = queue
            .lease(
                &mut con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        else {
            continue;
        };

        let Some(task) = CommitmentMapperAccumulatorTask::deserialize(&queue_item.data) else {
            println!("{}", "Invalid task data".red().bold());
            println!("{}", format!("Got bytes: {:?}", queue_item.data).red());
            queue.complete(&mut con, &queue_item).await?;
            continue;
        };

        task.log();

        match task {
            CommitmentMapperAccumulatorTask::AppendValidatorAccumulatorProof(validator_index) => {
                let validator_accumulator_index = validator_index;
                let protocol = protocol.to_string();
                let key = format!(
                    "{}:{}:{}",
                    VALIDATOR_COMMITMENT_CONSTANTS
                        .validator_accumulator_key
                        .to_owned(),
                    protocol,
                    validator_accumulator_index
                );

                let maybe_input =
                    fetch_redis_json_object::<ValidatorAccumulatorInput>(&mut con, key).await;

                match maybe_input {
                    Ok(validator) => {
                        let mut pw = PartialWitness::new();

                        pw.set_bytes_array(
                            &validator_commitment.validator_pubkey,
                            &hex::decode(format_hex(validator.validator_pubkey)).unwrap(),
                        );
                        pw.set_biguint_target(
                            &validator_commitment.validator_eth1_deposit_index,
                            &BigUint::from(validator.validator_eth1_deposit_index),
                        );
                        pw.set_bool_target(
                            validator_commitment.validator_is_zero,
                            validator_index == VALIDATOR_REGISTRY_LIMIT as u64,
                        );

                        let proof = first_level_circuit_data.prove(pw)?;

                        if validator_index as usize != VALIDATOR_REGISTRY_LIMIT {
                            match save_validator_accumulator_proof(
                                &mut con,
                                proof,
                                protocol.to_string(),
                                gindex_from_validator_index(validator_index, 32),
                            )
                            .await
                            {
                                Ok(_) => {
                                    queue.complete(&mut con, &queue_item).await?;
                                }
                                Err(err) => {
                                    println!(
                                        "{}",
                                        format!("Error while proving zero validator: {}", err)
                                            .red()
                                            .bold()
                                    );
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        } else {
                            match save_zero_validator_accumulator_proof(&mut con, proof, 32).await {
                                Ok(_) => {
                                    queue.complete(&mut con, &queue_item).await?;
                                }
                                Err(err) => {
                                    println!(
                                        "{}",
                                        format!("Error while proving validator: {}", err)
                                            .red()
                                            .bold()
                                    );
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!(
                            "{}",
                            format!("Error while fetching validator: {}", err)
                                .red()
                                .bold()
                        );
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
            }
            CommitmentMapperAccumulatorTask::UpdateProofNode(gindex) => {
                let level = 31 - get_depth_for_gindex(gindex) as usize;

                match fetch_accumulator_proofs(
                    &mut con,
                    proof_storage.as_mut(),
                    protocol.to_string(),
                    gindex,
                )
                .await
                {
                    Ok(proofs) => {
                        let inner_circuit_data = if level > 0 {
                            &inner_circuits[level - 1].1
                        } else {
                            &first_level_circuit_data
                        };

                        let proof = handle_commitment_mapper_inner_level_proof(
                            proofs.0,
                            proofs.1,
                            inner_circuit_data,
                            &inner_circuits[level].0,
                            &inner_circuits[level].1,
                        )?;

                        match save_validator_accumulator_proof(
                            &mut con,
                            proof,
                            protocol.to_string(),
                            gindex,
                        )
                        .await
                        {
                            Ok(_) => queue.complete(&mut con, &queue_item).await?,
                            Err(err) => {
                                println!(
                                    "{}",
                                    format!("Error while saving validator proof: {}", err)
                                        .red()
                                        .bold()
                                );
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!(
                            "{}",
                            format!("Error while fetching validator proof: {}", err)
                                .red()
                                .bold()
                        );
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
            CommitmentMapperAccumulatorTask::ProveZeroForDepth(depth) => {
                // the level in the inner proofs tree
                let level = 31 - depth as usize;

                match fetch_zero_accumulator_proof(&mut con, depth + 1).await {
                    Ok(lower_proof) => {
                        let inner_circuit_data = if level > 0 {
                            &inner_circuits[level - 1].1
                        } else {
                            &first_level_circuit_data
                        };

                        let lower_proof_bytes = lower_proof.get_proof(proof_storage.as_mut()).await;

                        let proof = handle_commitment_mapper_inner_level_proof(
                            lower_proof_bytes.clone(),
                            lower_proof_bytes.clone(),
                            inner_circuit_data,
                            &inner_circuits[level].0,
                            &inner_circuits[level].1,
                        )?;

                        match save_zero_validator_accumulator_proof(&mut con, proof, depth).await {
                            Ok(_) => queue.complete(&mut con, &queue_item).await?,
                            Err(err) => {
                                println!(
                                    "{}",
                                    format!("Error while saving zero validator proof: {}", err)
                                        .red()
                                        .bold()
                                );
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!(
                            "{}",
                            format!("Error while proving zero for depth {}: {}", depth, err)
                                .red()
                                .bold()
                        );
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
        };
    }
}

fn get_inner_targets(i: usize) -> Result<CommitmentMapperInnerCircuitTargets> {
    let target_bytes = read_from_file(&format!("{}_{}.plonky2_targets", CIRCUIT_NAME, i))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(CommitmentMapperInnerCircuitTargets::read_targets(&mut target_buffer).unwrap())
}

fn get_first_level_targets() -> Result<ValidatorAccumulatorCommitmentTargets> {
    let target_bytes = read_from_file(&format!("{}_{}.plonky2_targets", CIRCUIT_NAME, 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(ValidatorAccumulatorCommitmentTargets::read_targets(&mut target_buffer).unwrap())
}
