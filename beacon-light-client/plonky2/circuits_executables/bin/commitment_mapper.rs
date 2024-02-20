use anyhow::Result;
use circuits::{
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    targets_serialization::ReadTargets, validator_commitment_mapper::ValidatorCommitmentTargets,
};
use circuits_executables::{
    commitment_mapper_task::{deserialize_task, CommitmentMapperTask},
    crud::{
        common::{
            fetch_proofs, fetch_validator, fetch_zero_proof, get_depth_for_gindex,
            load_circuit_data, read_from_file, save_validator_proof, save_zero_validator_proof,
            ProofProvider, ValidatorProof,
        },
        proof_storage::proof_storage::create_proof_storage,
    },
    provers::{handle_commitment_mapper_inner_level_proof, SetPWValues},
    utils::{gindex_from_validator_index, parse_config_file},
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use clap::{App, Arg};
use colored::Colorize;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::{PartialWitness, WitnessWrite},
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
    util::serialization::Buffer,
};

use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, println, thread, time::Duration};

use jemallocator::Jemalloc;

use serde_binary::binary_stream;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<()> {
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
    .arg(
        Arg::with_name("mock")
        .long("mock")
        .help("Sets mock mode")
        .takes_value(false)
        .default_value("false")
    )
    .arg(
        Arg::with_name("proof_storage_type")
            .long("proof-storage-type")
            .value_name("proof_storage_type")
            .help("Sets the type of proof storage")
            .takes_value(true)
            .required(true)
            .possible_values(&["redis", "file", "azure", "aws"])
    )
    .arg(
        Arg::with_name("folder_name")
            .long("folder-name")
            .value_name("folder_name")
            .help("Sets the name of the folder proofs will be stored in")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("azure_account")
            .long("azure-account-name")
            .value_name("azure_account")
            .help("Sets the name of the azure account")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("azure_container")
            .long("azure-container-name")
            .value_name("azure_container")
            .help("Sets the name of the azure container")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("aws_endpoint_url")
            .long("aws-endpoint-url")
            .value_name("aws_endpoint_url")
            .help("Sets the aws endpoint url")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("aws_region")
            .long("aws-region")
            .value_name("aws_region")
            .help("Sets the aws region")
            .takes_value(true)
    )
    .arg(
        Arg::with_name("aws_bucket_name")
            .long("aws-bucket-name")
            .value_name("aws_bucket_name")
            .help("Sets the aws bucket name")
            .takes_value(true)
    )
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

    let mock = matches.value_of("mock").unwrap().parse::<bool>().unwrap();

    let inner_proof_mock_binary = include_bytes!("../mock_data/inner_proof_mapper.mock");
    let proof_mock_binary = include_bytes!("../mock_data/proof_mapper.mock");

    loop {
        if !mock {
            println!("{}", "Waiting for task...".yellow());
        }

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

                        let proof = if mock {
                            serde_binary::from_slice(proof_mock_binary, binary_stream::Endian::Big)
                                .unwrap()
                        } else {
                            first_level_circuit_data.prove(pw)?
                        };

                        if validator_index as usize != VALIDATOR_REGISTRY_LIMIT {
                            match save_validator_proof(
                                &mut con,
                                proof_storage.as_mut(),
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

                        let proof = if mock {
                            let inner_proof_mock: ProofWithPublicInputs<
                                GoldilocksField,
                                PoseidonGoldilocksConfig,
                                2,
                            > = serde_binary::from_slice(
                                inner_proof_mock_binary,
                                binary_stream::Endian::Big,
                            )
                            .unwrap();
                            inner_proof_mock
                        } else {
                            handle_commitment_mapper_inner_level_proof(
                                proofs.0,
                                proofs.1,
                                inner_circuit_data,
                                &inner_circuits[level].0,
                                &inner_circuits[level].1,
                            )?
                        };

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
                    Ok(proof) => {
                        let inner_circuit_data = if level > 0 {
                            &inner_circuits[level - 1].1
                        } else {
                            &first_level_circuit_data
                        };

                        let proof = handle_commitment_mapper_inner_level_proof(
                            proof.get_proof(proof_storage.as_mut()).await,
                            proof.get_proof(proof_storage.as_mut()).await,
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
    let target_bytes = read_from_file(&format!("commitment_mapper_{}.plonky2_targets", i))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(CommitmentMapperInnerCircuitTargets::read_targets(&mut target_buffer).unwrap())
}

fn get_first_level_targets() -> Result<ValidatorCommitmentTargets> {
    let target_bytes = read_from_file(&format!("commitment_mapper_{}.plonky2_targets", 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(ValidatorCommitmentTargets::read_targets(&mut target_buffer).unwrap())
}
