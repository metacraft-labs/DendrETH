#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use circuit::{
    serde_circuit_target::deserialize_circuit_target, set_witness::SetWitness, CircuitTargetType,
    SerdeCircuitTarget,
};
use circuit_executables::{
    constants::{SERIALIZED_CIRCUITS_DIR, VALIDATOR_REGISTRY_LIMIT},
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
};
use circuits::{
    common_targets::BasicRecursiveInnerCircuitTarget,
    types::BalanceProof,
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

const CIRCUIT_NAME: &str = "bls_verification";

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let common_config = parse_config_file("../../common_config.json".to_owned()).unwrap();

    let matches = CommandLineOptionsBuilder::new("bls_verification")
        .with_redis_options(&common_config.redis_host, common_config.redis_port)
        .with_work_queue_options()
        .with_proof_storage_options()
        .get_matches();

    let config = parse_balance_verification_command_line_options(&matches);

    println!("{}", "Connecting to Redis...".yellow());
    let client = redis::Client::open(config.redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let mut proof_storage = create_proof_storage(&matches).await;

    println!("{}", "Loading circuit data...".yellow());
    let circuit_data = load_circuit_data(&format!(
        "{}/{}_{}",
        SERIALIZED_CIRCUITS_DIR, CIRCUIT_NAME, &config.circuit_level
    ))?;

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
        start,
        config.time_to_run,
        config.stop_after,
        config.lease_for,
        &protocol,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    protocol: &str,
) -> Result<()>
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
    }

    Ok(())
}
