#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::{
    serde_circuit_target::deserialize_circuit_target, set_witness::SetWitness, CircuitTargetType,
    SerdeCircuitTarget,
};
use circuit_executables::{
    constants::VALIDATOR_REGISTRY_LIMIT,
    crud::{
        common::{
            delete_balance_verification_proof_dependencies, fetch_proofs_balances,
            fetch_validator_balance_input, load_circuit_data, read_from_file, save_balance_proof,
        },
        proof_storage::proof_storage::RedisBlobStorage,
    },
    db_constants::DB_CONSTANTS,
    provers::prove_inner_level,
    utils::{parse_balance_verification_command_line_options, CommandLineOptionsBuilder},
};
use circuits::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    redis_storage_types::WithdrawalCredentialsBalanceVerificationProofData,
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

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};

use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CommandLineOptionsBuilder::new("balance_verification")
        .with_balance_verification_options()
        .with_work_queue_options()
        .with_serialized_circuits_dir()
        .with_proof_storage_config()
        .get_matches();

    let serialized_circuits_dir = matches.value_of("serialized_circuits_dir").unwrap();
    let storage_config_filepath = matches.value_of("proof_storage_cfg").unwrap();

    let config = parse_balance_verification_command_line_options(&matches);

    println!("{}", "Initializing storage connection...".yellow());
    let mut storage =
        RedisBlobStorage::from_file(storage_config_filepath, "balance_verification").await?;

    println!("{}", "Loading circuit data...".yellow());
    let circuit_data = load_circuit_data::<WithdrawalCredentialsBalanceAggregatorFirstLevel<8, 1>>(
        serialized_circuits_dir,
        &format!("{}_{}", CIRCUIT_NAME, &config.circuit_level),
    )?;

    let (inner_circuit_data, targets) = if config.circuit_level == 0 {
        (
            None,
            get_first_level_targets::<8, 1>(serialized_circuits_dir)?,
        )
    } else {
        (
            Some(load_circuit_data::<
                WithdrawalCredentialsBalanceAggregatorFirstLevel<1, 8>,
            >(
                serialized_circuits_dir,
                &format!("{}_{}", CIRCUIT_NAME, config.circuit_level - 1),
            )?),
            get_inner_level_targets::<8, 1>(serialized_circuits_dir, config.circuit_level)?,
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
        &mut storage,
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
    storage: &mut RedisBlobStorage,
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
                &mut storage.metadata,
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
            queue.complete(&mut storage.metadata, &queue_item).await?;

            continue;
        }

        match targets {
            Targets::FirstLevel(targets) => {
                match process_first_level_task(
                    storage,
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
                    storage,
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
    storage: &mut RedisBlobStorage,
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
    let validator_balance_input = fetch_validator_balance_input(
        &mut storage.metadata,
        protocol.to_owned(),
        balance_input_index,
    )
    .await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let mut pw = PartialWitness::new();
    targets.set_witness(&mut pw, &validator_balance_input);
    let proof = circuit_data.prove(pw)?;

    match save_balance_proof::<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>(
        &mut storage.metadata,
        storage.blob.as_mut(),
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
            queue.complete(&mut storage.metadata, &queue_item).await?;
        }
    }

    Ok(())
}

async fn process_inner_level_job<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    storage: &mut RedisBlobStorage,
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

    match fetch_proofs_balances::<
        WithdrawalCredentialsBalanceVerificationProofData<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >,
    >(
        &mut storage.metadata,
        storage.blob.as_mut(),
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
                &mut storage.metadata,
                storage.blob.as_mut(),
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
                    queue.complete(&mut storage.metadata, &queue_item).await?;
                    if !preserve_intermediary_proofs {
                        // delete child nodes
                        delete_balance_verification_proof_dependencies(
                            &mut storage.metadata,
                            storage.blob.as_mut(),
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
>(
    serialized_circuits_dir: &str,
) -> Result<Targets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>, anyhow::Error>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let target_bytes = read_from_file(&format!(
        "{}/{}_0.plonky2_targets",
        serialized_circuits_dir, CIRCUIT_NAME
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
    serialized_circuits_dir: &str,
    level: u64,
) -> Result<Targets<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        serialized_circuits_dir, CIRCUIT_NAME, level
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
