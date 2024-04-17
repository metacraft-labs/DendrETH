use circuits_executables::{
    crud::{
        common::{
            delete_balance_verification_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_input, load_circuit_data, read_from_file, save_balance_proof,
            BalanceProof,
        },
        proof_storage::proof_storage::{create_proof_storage, ProofStorage},
    },
    provers::{handle_balance_inner_level_proof, SetPWValues},
    utils::{
        parse_balance_verification_command_line_options, parse_config_file,
        CommandLineOptionsBuilder,
    },
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use colored::Colorize;
use std::{
    println, thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use circuits::{
    build_balance_inner_level_circuit::BalanceInnerCircuitTargets,
    targets_serialization::ReadTargets,
    validator_balance_circuit::ValidatorBalanceVerificationTargets,
};

use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "balance_verification";

enum Targets<const N: usize> {
    FirstLevel(Option<ValidatorBalanceVerificationTargets<N>>),
    InnerLevel(Option<BalanceInnerCircuitTargets>),
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("balance_verification")
        .with_balance_verification_options()
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
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
    let circuit_data = load_circuit_data(&format!(
        "{}/{}_{}",
        CIRCUIT_DIR, CIRCUIT_NAME, &config.circuit_level
    ))?;

    let (inner_circuit_data, targets) = if config.circuit_level == 0 {
        (None, get_first_level_targets::<1>()?)
    } else {
        (
            Some(load_circuit_data(&format!(
                "{}/{}_{}",
                CIRCUIT_DIR,
                CIRCUIT_NAME,
                config.circuit_level - 1
            ))?),
            get_inner_level_targets::<1>(config.circuit_level)?,
        )
    };

    println!(
        "{}",
        format!("Starting worker for level {}...", config.circuit_level).yellow()
    );

    let protocol = matches.value_of("protocol").unwrap();

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}:{}",
        protocol, VALIDATOR_COMMITMENT_CONSTANTS.balance_verification_queue, config.circuit_level
    )));

    let start: Instant = Instant::now();
    process_queue(
        &mut con,
        proof_storage.as_mut(),
        &queue,
        &circuit_data,
        inner_circuit_data.as_ref(),
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

async fn process_queue<const N: usize>(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets<N>,
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
                match process_inner_level_job::<N>(
                    con,
                    proof_storage,
                    queue,
                    queue_item,
                    circuit_data,
                    inner_circuit_data.unwrap(),
                    inner_circuit_targets,
                    level,
                    preserve_intermediary_proofs,
                    protocol.clone(),
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

async fn process_first_level_task<const N: usize>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &ValidatorBalanceVerificationTargets<N>,
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
        fetch_validator_balance_input::<N>(con, protocol.to_owned(), balance_input_index).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let mut pw = PartialWitness::new();

    targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = circuit_data.prove(pw)?;

    match save_balance_proof::<N>(
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

async fn process_inner_level_job<const N: usize>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &Option<BalanceInnerCircuitTargets>,
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

    match fetch_proofs_balances::<BalanceProof>(
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
            let proof = handle_balance_inner_level_proof(
                proofs.0,
                proofs.1,
                &inner_circuit_data,
                &inner_circuit_targets.as_ref().unwrap(),
                &circuit_data,
            )?;

            match save_balance_proof::<N>(
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
                        delete_balance_verification_proof_dependencies(
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

fn get_first_level_targets<const N: usize>() -> Result<Targets<N>, anyhow::Error> {
    let target_bytes = read_from_file(&format!(
        "{}/{}_0.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::FirstLevel(Some(
        ValidatorBalanceVerificationTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}

fn get_inner_level_targets<const N: usize>(level: u64) -> Result<Targets<N>> {
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME, level
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::<N>::InnerLevel(Some(
        BalanceInnerCircuitTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}
