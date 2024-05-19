#![feature(generic_const_exprs)]
use circuit::{
    serde_circuit_target::deserialize_circuit_target, set_witness::SetWitness, Circuit,
    CircuitTargetType, SerdeCircuitTarget,
};
use circuit_executables::{
    crud::{
        common::{
            delete_balance_verification_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_input, load_circuit_data, read_from_file, save_balance_proof,
        },
        proof_storage::proof_storage::{create_proof_storage, ProofStorage},
    },
    db_constants::DB_CONSTANTS,
    provers::prove_inner_level,
    utils::{
        parse_balance_verification_command_line_options, parse_config_file,
        CommandLineOptionsBuilder,
    },
    validator::VALIDATOR_REGISTRY_LIMIT,
};
use circuits::{
    circuit_input_common::BalanceProof,
    common_targets::BasicRecursiveInnerCircuitTarget,
    withdrawal_credentials_balance_aggregator::{
        first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
        inner_level::WithdrawalCredentialsBalanceAggregatorInnerLevel,
    },
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
    util::serialization::Buffer,
};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "balance_verification";

enum Targets<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    FirstLevel(
        Option<
            CircuitTargetType<
                WithdrawalCredentialsBalanceAggregatorFirstLevel<
                    VALIDATORS_COUNT,
                    WITHDRAWAL_CREDENTIALS_COUNT,
                >,
            >,
        >,
    ),
    InnerLevel(Option<BasicRecursiveInnerCircuitTarget>),
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

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
        (None, get_first_level_targets::<8, 1>()?)
    } else {
        (
            Some(load_circuit_data(&format!(
                "{}/{}_{}",
                CIRCUIT_DIR,
                CIRCUIT_NAME,
                config.circuit_level - 1
            ))?),
            get_inner_level_targets::<8, 1>(config.circuit_level)?,
        )
    };

    println!(
        "{}",
        format!("Starting worker for level {}...", config.circuit_level).yellow()
    );

    let protocol = matches.value_of("protocol").unwrap();

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}:{}",
        protocol, DB_CONSTANTS.balance_verification_queue, config.circuit_level
    )));

    println!(
        "{}",
        &format!(
            "{}:{}:{}",
            protocol, DB_CONSTANTS.balance_verification_queue, config.circuit_level
        )
    );

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

async fn process_queue<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>,
    level: u64,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    preserve_intermediary_proofs: bool,
    protocol: &str,
) -> Result<()>
where
    [(); VALIDATORS_COUNT / 4]:,
{
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
                match process_inner_level_job::<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>(
                    con,
                    proof_storage,
                    queue,
                    queue_item,
                    circuit_data,
                    inner_circuit_data.unwrap(),
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

async fn process_first_level_task<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &CircuitTargetType<
        WithdrawalCredentialsBalanceAggregatorFirstLevel<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >,
    >,
    protocol: &str,
) -> Result<()>
where
    [(); VALIDATORS_COUNT / 4]:,
{
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
        fetch_validator_balance_input(con, protocol.to_owned(), balance_input_index).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    println!(
        "witness input: {}\n",
        serde_json::to_string(&validator_balance_input).unwrap()
    );

    let mut pw = PartialWitness::new();
    targets.set_witness(&mut pw, &validator_balance_input);

    let proof = circuit_data.prove(pw)?;

    let pis = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs(&proof.public_inputs);

    println!("pis: {}", serde_json::to_string(&pis).unwrap());

    match save_balance_proof::<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>(
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

async fn process_inner_level_job<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
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
) -> Result<()>
where
    [(); VALIDATORS_COUNT / 4]:,
{
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

    match fetch_proofs_balances::<BalanceProof<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>>(
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

            match save_balance_proof::<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>(
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

fn get_first_level_targets<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>() -> Result<Targets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>, anyhow::Error>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let target_bytes = read_from_file(&format!(
        "{}/{}_0.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::FirstLevel(Some(
        deserialize_circuit_target::<
            WithdrawalCredentialsBalanceAggregatorFirstLevel<
                VALIDATORS_COUNT,
                WITHDRAWAL_CREDENTIALS_COUNT,
            >,
        >(&mut target_buffer)
        .unwrap(),
    )))
}

fn get_inner_level_targets<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    level: u64,
) -> Result<Targets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME, level
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(
        Targets::<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>::InnerLevel(Some(
            CircuitTargetType::<
                WithdrawalCredentialsBalanceAggregatorInnerLevel<
                    VALIDATORS_COUNT,
                    WITHDRAWAL_CREDENTIALS_COUNT,
                >,
            >::deserialize(&mut target_buffer)
            .unwrap(),
        )),
    )
}
