#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::{set_witness::SetWitness, Circuit, CircuitTargetType};
use circuit_executables::{
    cached_circuit_build::{build_recursive_circuit_single_level_cached, SERIALIZED_CIRCUITS_DIR},
    constants::VALIDATOR_REGISTRY_LIMIT,
    crud::{
        common::{
            delete_balance_verification_diva_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_aggregator_input, load_circuit_data,
            save_balance_aggregator_proof,
        },
        proof_storage::proof_storage::{create_proof_storage, ProofStorage},
    },
    db_constants::DB_CONSTANTS,
    provers::prove_inner_level,
    utils::{
        parse_balance_verification_command_line_options, parse_config_file,
        CommandLineOptionsBuilder,
    },
};
use circuits::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    deposit_accumulator_balance_aggregator_diva::{
        first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
        inner_level::DepositAccumulatorBalanceAggregatorDivaInnerLevel,
    },
    redis_storage_types::DepositAccumulatorBalanceAggregatorDivaProofData,
};
use colored::Colorize;
use std::{
    println, thread,
    time::{Duration, Instant},
};

use anyhow::Result;

use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_NAME: &str = "deposit_accumulator_balance_aggregator_diva";

enum Targets {
    FirstLevel(Option<CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>>),
    InnerLevel(Option<BasicRecursiveInnerCircuitTarget>),
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("balance_verification")
        .with_balance_verification_options()
        .with_redis_options(&common_config.redis_host, common_config.redis_port, &common_config.redis_auth)
        .with_work_queue_options()
        .with_proof_storage_options()
        .with_protocol_options()
        .get_matches();

    let config = parse_balance_verification_command_line_options(&matches);

    println!("{}", "Connecting to Redis...".yellow());
    let client = redis::Client::open(config.redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let mut proof_storage = create_proof_storage(&matches).await;

    println!("{}", "Loading circuit data...".yellow());

    let inner_circuit_data = if config.circuit_level == 0 {
        None
    } else {
        Some(
            load_circuit_data(&format!(
                "{}/{}_{}",
                SERIALIZED_CIRCUITS_DIR,
                CIRCUIT_NAME,
                &config.circuit_level - 1
            ))
            .unwrap(),
        )
    };

    let (targets, circuit_data) = match inner_circuit_data {
        Some(ref inner_circuit_data) => {
            let circuit = build_recursive_circuit_single_level_cached(
                CIRCUIT_NAME,
                config.circuit_level as usize,
                &|| DepositAccumulatorBalanceAggregatorDivaInnerLevel::build(&inner_circuit_data),
            );

            (Targets::InnerLevel(Some(circuit.0)), circuit.1)
        }
        None => {
            let circuit = build_recursive_circuit_single_level_cached(
                CIRCUIT_NAME,
                config.circuit_level as usize,
                &|| DepositAccumulatorBalanceAggregatorDivaFirstLevel::build(&()),
            );

            (Targets::FirstLevel(Some(circuit.0)), circuit.1)
        }
    };

    println!(
        "{}",
        format!("Starting worker for level {}...", config.circuit_level).yellow()
    );

    let protocol = matches.value_of("protocol").unwrap();

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}:{}",
        protocol, DB_CONSTANTS.deposit_balance_verification_queue, config.circuit_level
    )));

    println!(
        "{}",
        &format!(
            "{}:{}:{}",
            protocol, DB_CONSTANTS.deposit_balance_verification_queue, config.circuit_level
        )
    );

    let start: Instant = Instant::now();
    process_queue(
        &mut con,
        proof_storage.as_mut(),
        &queue,
        &circuit_data,
        &inner_circuit_data,
        &targets,
        config.circuit_level,
        start,
        config.time_to_run,
        config.stop_after,
        config.lease_for,
        config.preserve_intermediary_proofs,
        &protocol,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &Option<CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    level: u64,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) -> Result<()> {
    while time_to_run.is_none() || start.elapsed() < time_to_run.unwrap() {
        let queue_item = match queue
            .lease(
                con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(item) => item,
            None => {
                println!("{}", "No tasks left in queue".bright_green().bold());

                return Ok(());
            }
        };

        if queue_item.data.is_empty() {
            println!("{}", "Skipping empty data task".yellow());
            queue.complete(con, &queue_item).await?;

            continue;
        }

        match targets {
            Targets::FirstLevel(targets) => {
                match process_first_level_task(
                    con,
                    proof_storage,
                    queue,
                    queue_item,
                    circuit_data,
                    targets.as_ref().unwrap(),
                    protocol,
                )
                .await
                {
                    Err(err) => {
                        println!(
                            "{}",
                            format!("Error processing first level task {:?}", err)
                                .red()
                                .bold()
                        );
                        continue;
                    }
                    Ok(_) => {}
                };
            }
            Targets::InnerLevel(inner_circuit_targets) => {
                match process_inner_level_job(
                    con,
                    proof_storage,
                    queue,
                    queue_item,
                    circuit_data,
                    inner_circuit_data.as_ref().unwrap(),
                    inner_circuit_targets,
                    level,
                    preserve_intermediary_proofs,
                    protocol,
                )
                .await
                {
                    Err(err) => {
                        println!(
                            "{}",
                            format!("Error processing inner level task {:?}", err)
                                .red()
                                .bold()
                        );
                        continue;
                    }
                    Ok(_) => {}
                };
            }
        }
    }

    Ok(())
}

async fn process_first_level_task(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &CircuitTargetType<DepositAccumulatorBalanceAggregatorDivaFirstLevel>,
    protocol: &str,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(queue_item.data[0..8].try_into().unwrap());

    if balance_input_index as usize != VALIDATOR_REGISTRY_LIMIT {
        println!(
            "{}",
            format!(
                "Processing task for index {}...",
                balance_input_index.to_string().magenta()
            )
            .blue()
            .bold()
        );
    } else {
        println!("{}", "Processing task for zero proof...".blue().bold());
    }

    let start = Instant::now();
    let validator_balance_input =
        fetch_validator_balance_aggregator_input(con, protocol.to_owned(), balance_input_index)
            .await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let mut pw = PartialWitness::new();
    targets.set_witness(&mut pw, &validator_balance_input);
    let proof = circuit_data.prove(pw)?;

    match save_balance_aggregator_proof(
        con,
        proof_storage,
        protocol.to_owned(),
        proof,
        0,
        balance_input_index,
    )
    .await
    {
        Err(err) => {
            println!(
                "{}",
                format!("Error while saving balance proof: {}", err)
                    .red()
                    .bold()
            );
            thread::sleep(Duration::from_secs(5));
            return Err(err);
        }
        Ok(_) => {
            queue.complete(con, &queue_item).await?;
        }
    }

    Ok(())
}

async fn process_inner_level_job(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_target: &Option<BasicRecursiveInnerCircuitTarget>,
    level: u64,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) -> Result<()> {
    let index = u64::from_be_bytes(queue_item.data[0..8].try_into().unwrap());

    if index as usize != VALIDATOR_REGISTRY_LIMIT {
        println!(
            "{}",
            format!(
                "Processing task for index {}...",
                index.to_string().magenta()
            )
            .blue()
            .bold()
        );
    } else {
        println!("{}", "Processing task for zero proof...".blue().bold());
    }

    match fetch_proofs_balances::<DepositAccumulatorBalanceAggregatorDivaProofData>(
        con,
        proof_storage,
        protocol.to_owned(),
        level,
        index,
    )
    .await
    {
        Err(err) => {
            println!(
                "{}",
                format!("Error while fetching balance proofs: {}", err)
                    .red()
                    .bold()
            );
            return Err(err);
        }
        Ok(proofs) => {
            let proof = prove_inner_level(
                proofs.0,
                proofs.1,
                &inner_circuit_data,
                &inner_circuit_target.as_ref().unwrap(),
                &circuit_data,
            )?;

            match save_balance_aggregator_proof(
                con,
                proof_storage,
                protocol.to_owned(),
                proof,
                level,
                index,
            )
            .await
            {
                Err(err) => {
                    println!(
                        "{}",
                        format!("Error while saving balance proof: {}", err)
                            .red()
                            .bold()
                    );
                    thread::sleep(Duration::from_secs(5));
                    return Err(err);
                }
                Ok(_) => {
                    queue.complete(con, &queue_item).await?;
                    if !preserve_intermediary_proofs {
                        // delete child nodes
                        delete_balance_verification_diva_proof_dependencies(
                            con,
                            proof_storage,
                            protocol,
                            level,
                            index,
                        )
                        .await?;
                    }
                }
            }
            Ok(())
        }
    }
}
