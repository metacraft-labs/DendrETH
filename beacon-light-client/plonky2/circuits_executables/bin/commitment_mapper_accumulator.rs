use anyhow::Result;
use circuits::{
    biguint::WitnessBigUint,
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    targets_serialization::ReadTargets, utils::SetBytesArray,
    validator_accumulator_commitment_mapper::ValidatorAccumulatorCommitmentTargets,
};
use circuits_executables::{
    crud::{
        fetch_accumulator_proofs, fetch_validator_accumulator, fetch_zero_accumulator_proof,
        get_depth_for_gindex, load_circuit_data, read_from_file, save_validator_accumulator_proof,
        save_zero_validator_accumulator_proof, ProofProvider, ValidatorProof,
    },
    provers::handle_commitment_mapper_inner_level_proof,
    validator::VALIDATOR_REGISTRY_LIMIT, validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use clap::{App, Arg};
use futures_lite::future;
use num::FromPrimitive;
use num_derive::FromPrimitive;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, println, thread, time::Duration};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "commitment_mapper_accumulator";

#[derive(FromPrimitive)]
#[repr(u8)]
enum CommitmentMapperAccumulatorTaskType {
    UpdateProofNode,
    ProveZeroForLevel,
    UpdateValidatorProof,
    None,
}

#[derive(Debug)]
enum CommitmentMapperAccumulatorTask {
    UpdateProofNode(u64),      // gindex
    ProveZeroForLevel(u64),    // level
    UpdateValidatorProof(u64), // validator index
    None,
}

fn deserialize_task(bytes: &[u8]) -> CommitmentMapperAccumulatorTask {
    let task_type: Option<CommitmentMapperAccumulatorTaskType> =
        FromPrimitive::from_u8(u8::from_be_bytes(bytes[0..1].try_into().unwrap()));

    if task_type.is_none() {
        return CommitmentMapperAccumulatorTask::None;
    }

    let task_type = task_type.unwrap();

    match task_type {
        CommitmentMapperAccumulatorTaskType::UpdateProofNode => {
            let gindex = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            CommitmentMapperAccumulatorTask::UpdateProofNode(gindex)
        }
        CommitmentMapperAccumulatorTaskType::ProveZeroForLevel => {
            let level = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            CommitmentMapperAccumulatorTask::ProveZeroForLevel(level)
        }
        CommitmentMapperAccumulatorTaskType::UpdateValidatorProof => {
            let validator_index = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            CommitmentMapperAccumulatorTask::UpdateValidatorProof(validator_index)
        }
        CommitmentMapperAccumulatorTaskType::None => unreachable!(),
    }
}

fn gindex_from_validator_index(index: u64) -> u64 {
    return 2u64.pow(40) - 1 + index;
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let matches = App::new("")
    .arg(
        Arg::with_name("redis_connection")
            .short('r')
            .long("redis")
            .value_name("Redis Connection")
            .help("Sets a custom Redis connection")
            .takes_value(true)
            .default_value("redis://127.0.0.1:6379/"),
    )
    .arg(
        Arg::with_name("stop_after")
        .long("stop-after")
        .value_name("Stop after")
        .help("Sets how much seconds to wait until the program stops if no new tasks are found in the queue")
        .takes_value(true)
        .default_value("20")
    )
    .arg(
        Arg::with_name("lease_for")
        .value_name("lease-for")
        .help("Sets for how long the task will be leased and then possibly requeued if not finished")
        .takes_value(true)
        .default_value("30"))
    .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

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

    for i in 1..41 {
        inner_circuits.push((
            get_inner_targets(i)?,
            load_circuit_data(&format!("{}_{}", CIRCUIT_NAME, i))?,
        ));
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
        println!("Waiting for job...");

        let job = match queue
            .lease(
                &mut con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(job) => job,
            None => continue,
        };

        println!("Got job: {:?}", job.data);

        let task = deserialize_task(&job.data);

        match task {
            CommitmentMapperAccumulatorTask::UpdateValidatorProof(validator_index) => {
                match fetch_validator_accumulator(&mut con, validator_index).await {
                    Ok(validator) => {
                        let mut pw = PartialWitness::new();

                        pw.set_bytes_array(
                            &validator_commitment.validator_pubkey,
                            &hex::decode(validator.validator_pubkey).unwrap(),
                        );
                        pw.set_biguint_target(
                            &validator_commitment.validator_eth1_deposit_index,
                            &validator.validator_eth1_deposit_index,
                        );

                        let proof = first_level_circuit_data.prove(pw)?;

                        if validator_index as usize != VALIDATOR_REGISTRY_LIMIT {
                            match save_validator_accumulator_proof(
                                &mut con,
                                proof,
                                gindex_from_validator_index(validator_index),
                            )
                            .await
                            {
                                Ok(_) => {
                                    queue.complete(&mut con, &job).await?;
                                }
                                Err(err) => {
                                    println!("Error: {}", err);
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        } else {
                            match save_zero_validator_accumulator_proof(&mut con, proof, 40).await {
                                Ok(_) => {
                                    queue.complete(&mut con, &job).await?;
                                }
                                Err(err) => {
                                    println!("Error: {}", err);
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
            }
            CommitmentMapperAccumulatorTask::UpdateProofNode(gindex) => {
                let level = 39 - get_depth_for_gindex(gindex) as usize;

                match fetch_accumulator_proofs(&mut con, gindex).await {
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

                        match save_validator_accumulator_proof(&mut con, proof, gindex).await {
                            Ok(_) => queue.complete(&mut con, &job).await?,
                            Err(err) => {
                                println!("Error: {}", err);
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
            CommitmentMapperAccumulatorTask::ProveZeroForLevel(depth) => {
                println!("Got prove zero level task");
                // the level in the inner proofs tree
                let level = 39 - depth as usize;

                match fetch_zero_accumulator_proof::<ValidatorProof>(&mut con, depth + 1).await {
                    Ok(proof) => {
                        let inner_circuit_data = if level > 0 {
                            &inner_circuits[level - 1].1
                        } else {
                            &first_level_circuit_data
                        };

                        let proof = handle_commitment_mapper_inner_level_proof(
                            proof.get_proof(),
                            proof.get_proof(),
                            inner_circuit_data,
                            &inner_circuits[level].0,
                            &inner_circuits[level].1,
                        )?;

                        match save_zero_validator_accumulator_proof(&mut con, proof, depth).await {
                            Ok(_) => queue.complete(&mut con, &job).await?,
                            Err(err) => {
                                println!("Error: {}", err);
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
            _ => {
                println!("Invalid job data");
                println!("This is bug from somewhere");

                queue.complete(&mut con, &job).await?;
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
