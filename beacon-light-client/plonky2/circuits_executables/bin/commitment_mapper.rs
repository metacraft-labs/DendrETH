use anyhow::Result;
use circuits::{
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    targets_serialization::ReadTargets, validator_commitment_mapper::ValidatorCommitmentTargets,
};
use circuits_executables::{
    commitment_mapper_task::CommitmentMapperTask,
    crud::{
        common::{
            fetch_proofs, fetch_validator, fetch_zero_proof, load_circuit_data, read_from_file,
            save_validator_proof, save_zero_validator_proof, ProofProvider, ValidatorProof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    provers::{handle_commitment_mapper_inner_level_proof, SetPWValues},
    utils::{
        get_depth_for_gindex, gindex_from_validator_index, parse_config_file,
        CommandLineOptionsBuilder,
    },
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use colored::Colorize;
use futures_lite::future;
use jemallocator::Jemalloc;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, println, thread, time::Duration};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "commitment_mapper";

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let config = parse_config_file("../common_config.json".to_owned())?;

    let matches = CommandLineOptionsBuilder::new("commitment_mapper")
        .with_redis_options(&config.redis_host, config.redis_port)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let mut proof_storage = create_proof_storage(&matches).await;

    println!("Connected to redis");

    let queue = WorkQueue::new(KeyPrefix::new(
        VALIDATOR_COMMITMENT_CONSTANTS
            .validator_proofs_queue
            .to_owned(),
    ));

    let first_level_circuit_data =
        load_circuit_data(&format!("{}/{}_0", CIRCUIT_DIR, CIRCUIT_NAME))?;
    let validator_commitment = get_first_level_targets()?;

    let mut inner_circuits: Vec<(
        CommitmentMapperInnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::new();

    for i in 1..41 {
        inner_circuits.push((
            get_inner_targets(i)?,
            load_circuit_data(&format!("{}/{}_{}", CIRCUIT_DIR, CIRCUIT_NAME, i))?,
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

        let Some(task) = CommitmentMapperTask::deserialize(&queue_item.data) else {
            println!("{}", "Invalid task data".red().bold());
            println!("{}", format!("Got bytes: {:?}", queue_item.data).red());
            queue.complete(&mut con, &queue_item).await?;
            continue;
        };

        task.log();

        match task {
            CommitmentMapperTask::UpdateValidatorProof(validator_index, epoch) => {
                match fetch_validator(&mut con, validator_index, epoch).await {
                    Ok(validator) => {
                        let mut pw = PartialWitness::new();

                        validator_commitment
                            .validator
                            .set_pw_values(&mut pw, &validator);

                        pw.set_bool_target(
                            validator_commitment.validator_is_zero,
                            validator_index == VALIDATOR_REGISTRY_LIMIT as u64,
                        );

                        let proof = first_level_circuit_data.prove(pw)?;

                        if validator_index as usize != VALIDATOR_REGISTRY_LIMIT {
                            match save_validator_proof(
                                &mut con,
                                proof_storage.as_mut(),
                                proof,
                                gindex_from_validator_index(validator_index, 40),
                                epoch,
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
                            match save_zero_validator_proof(
                                &mut con,
                                proof_storage.as_mut(),
                                proof,
                                40,
                            )
                            .await
                            {
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
            CommitmentMapperTask::UpdateProofNode(gindex, epoch) => {
                let level = 39 - get_depth_for_gindex(gindex) as usize;

                match fetch_proofs::<ValidatorProof>(
                    &mut con,
                    proof_storage.as_mut(),
                    gindex,
                    epoch,
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

                        if let Err(err) = save_validator_proof(
                            &mut con,
                            proof_storage.as_mut(),
                            proof,
                            gindex,
                            epoch,
                        )
                        .await
                        {
                            println!(
                                "{}",
                                format!("Error while saving validator proof: {}", err)
                                    .red()
                                    .bold()
                            );
                            thread::sleep(Duration::from_secs(1));
                            continue;
                        } else {
                            queue.complete(&mut con, &queue_item).await?
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
            CommitmentMapperTask::ProveZeroForDepth(depth) => {
                // the level in the inner proofs tree
                let level = 39 - depth as usize;

                match fetch_zero_proof::<ValidatorProof>(&mut con, depth + 1).await {
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

                        match save_zero_validator_proof(
                            &mut con,
                            proof_storage.as_mut(),
                            proof,
                            depth,
                        )
                        .await
                        {
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
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME, i
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(CommitmentMapperInnerCircuitTargets::read_targets(&mut target_buffer).unwrap())
}

fn get_first_level_targets() -> Result<ValidatorCommitmentTargets> {
    let target_bytes = read_from_file(&format!(
        "{}/{}_0.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(ValidatorCommitmentTargets::read_targets(&mut target_buffer).unwrap())
}
