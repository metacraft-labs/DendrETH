use anyhow::Result;
use circuits::{
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    targets_serialization::ReadTargets, validator_commitment_mapper::ValidatorCommitmentTargets,
};
use circuits_executables::{
    commitment_mapper_task::{deserialize_task, CommitmentMapperTask},
    crud::{
        fetch_proofs, fetch_validator, fetch_zero_proof, get_depth_for_gindex, load_circuit_data,
        read_from_file, save_validator_proof, save_zero_validator_proof, ProofProvider,
        ValidatorProof,
    },
    provers::{handle_commitment_mapper_inner_level_proof, SetPWValues},
    utils::{gindex_from_validator_index, parse_config_file},
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants,
};
use clap::{App, Arg};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, println, thread, time::Duration};

use validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let config = parse_config_file("../common_config.json".to_owned())?;

    let matches = App::new("")
    .arg(
        Arg::with_name("redis_connection")
            .short('r')
            .long("redis")
            .value_name("Redis Connection")
            .help("Sets a custom Redis connection")
            .takes_value(true)
            .default_value(&format!("redis://{}:{}/", config["redis-host"], config["redis-port"])),
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
            .validator_proofs_queue
            .to_owned(),
    ));

    let first_level_circuit_data = load_circuit_data("commitment_mapper_0")?;
    let validator_commitment = get_first_level_targets()?;

    let mut inner_circuits: Vec<(
        CommitmentMapperInnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::new();

    for i in 1..41 {
        inner_circuits.push((
            get_inner_targets(i)?,
            load_circuit_data(&format!("commitment_mapper_{}", i))?,
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
        println!("Waiting for task...");

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

        let Some(task) = deserialize_task(&queue_item.data) else {
            println!("Invalid task data");
            println!("Got bytes: {:?}", queue_item.data);
            println!("This is bug from somewhere");
            queue.complete(&mut con, &queue_item).await?;
            continue;
        };

        println!("Got task: {}", task);

        match task {
            CommitmentMapperTask::UpdateValidatorProof(validator_index, epoch) => {
                match fetch_validator(&mut con, validator_index, epoch).await {
                    Ok(validator) => {
                        let mut pw = PartialWitness::new();

                        validator_commitment
                            .validator
                            .set_pw_values(&mut pw, &validator);

                        let proof = first_level_circuit_data.prove(pw)?;

                        if validator_index as usize != VALIDATOR_REGISTRY_LIMIT {
                            match save_validator_proof(
                                &mut con,
                                proof,
                                gindex_from_validator_index(validator_index),
                                epoch,
                            )
                            .await
                            {
                                Ok(_) => {
                                    queue.complete(&mut con, &queue_item).await?;
                                }
                                Err(err) => {
                                    println!("Error while proving zero validator: {}", err);
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        } else {
                            match save_zero_validator_proof(&mut con, proof, 40).await {
                                Ok(_) => {
                                    queue.complete(&mut con, &queue_item).await?;
                                }
                                Err(err) => {
                                    println!("Error while proving validator: {}", err);
                                    thread::sleep(Duration::from_secs(10));
                                    continue;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error while fetching validator: {}", err);
                        thread::sleep(Duration::from_secs(10));
                        continue;
                    }
                };
            }
            CommitmentMapperTask::UpdateProofNode(gindex, epoch) => {
                let level = 39 - get_depth_for_gindex(gindex) as usize;

                match fetch_proofs::<ValidatorProof>(&mut con, gindex, epoch).await {
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
                            false,
                        )?;

                        match save_validator_proof(&mut con, proof, gindex, epoch).await {
                            Ok(_) => queue.complete(&mut con, &queue_item).await?,
                            Err(err) => {
                                println!("Error while saving validator proof: {}", err);
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error while fetching validator proof: {}", err);
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
            CommitmentMapperTask::ProveZeroForDepth(depth) => {
                // the level in the inner proofs tree
                let level = 39 - depth as usize;

                match fetch_zero_proof::<ValidatorProof>(&mut con, depth + 1).await {
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
                            false,
                        )?;

                        match save_zero_validator_proof(&mut con, proof, depth).await {
                            Ok(_) => queue.complete(&mut con, &queue_item).await?,
                            Err(err) => {
                                println!("Error while saving zero validator proof: {}", err);
                                thread::sleep(Duration::from_secs(1));
                                continue;
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error while proving zero for depth {}: {}", depth, err);
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };
            }
        };
    }
}

fn get_inner_targets(i: usize) -> Result<CommitmentMapperInnerCircuitTargets> {
    let target_bytes = read_from_file(&format!("commitment_mapper_{}.plonky2_targets", i))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(CommitmentMapperInnerCircuitTargets::read_targets(&mut target_buffer).unwrap())
}

fn get_first_level_targets() -> Result<ValidatorCommitmentTargets> {
    let target_bytes = read_from_file(&format!("commitment_mapper_{}.plonky2_targets", 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(ValidatorCommitmentTargets::read_targets(&mut target_buffer).unwrap())
}
