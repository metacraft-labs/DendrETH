use std::{
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use circuit_executables::{
    crud::common::{
        fetch_validator_balance_accumulator_input, load_circuit_data, read_from_file,
        save_balance_accumulator_proof,
    },
    provers::SetPWValues,
    utils::{
        parse_balance_verification_command_line_options, parse_config_file,
        CommandLineOptionsBuilder,
    },
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use circuits::{
    deposits_accumulator_balance_aggregator::validator_balance_circuit_accumulator::ValidatorBalanceVerificationAccumulatorTargets,
    serialization::targets_serialization::ReadTargets,
    withdrawal_credentials_balance_aggregator::build_balance_inner_level_circuit::BalanceInnerCircuitTargets,
};
use colored::Colorize;
use futures_lite::future;
use jemallocator::Jemalloc;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const CIRCUIT_DIR: &str = "circuits";
const CIRCUIT_NAME: &str = "balance_accumulator";

enum Targets {
    FirstLevel(Option<ValidatorBalanceVerificationAccumulatorTargets>),
    InnerLevel(Option<BalanceInnerCircuitTargets>),
}

fn bits_to_bytes(bits: &[u64]) -> Vec<u8> {
    bits.chunks(8)
        .map(|bits| (0..8usize).fold(0u8, |byte, pos| byte | ((bits[pos]) << (7 - pos)) as u8))
        .collect::<Vec<_>>()
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
        .get_matches();

    let config = parse_balance_verification_command_line_options(&matches);

    println!("{}", "Connecting to Redis...".yellow());
    let client = redis::Client::open(config.redis_connection)?;
    let mut con = client.get_async_connection().await?;

    println!("{}", "Loading circuit data...".yellow());

    let recursive_circuit = load_recursive_circuit_for_level(config.circuit_level)?;

    println!(
        "{}",
        format!("Starting worker for level {}...", config.circuit_level).yellow()
    );

    println!(
        "Debug {}",
        format!(
            "{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS.balance_verification_accumulator_proof_queue,
            config.circuit_level
        )
    );

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS.balance_verification_accumulator_proof_queue,
        config.circuit_level
    )));

    let start: Instant = Instant::now();

    process_queue(
        &mut con,
        &queue,
        &recursive_circuit.data,
        recursive_circuit.lower_circuit_data.as_ref(),
        &recursive_circuit.targets,
        config.circuit_level,
        config.protocol.unwrap(),
        start,
        config.time_to_run,
        config.stop_after,
        config.lease_for,
        config.preserve_intermediary_proofs,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    _inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    _level: u64,
    protocol: String,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    _preserve_intermediary_proofs: bool,
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
                    queue,
                    queue_item,
                    circuit_data,
                    protocol.clone(),
                    targets.as_ref().unwrap(),
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
            Targets::InnerLevel(_inner_circuit_targets) => {
                // match process_inner_level_job::<N>(
                //     con,
                //     queue,
                //     queue_item,
                //     circuit_data,
                //     inner_circuit_data.unwrap(),
                //     inner_circuit_targets,
                //     level,
                //     preserve_intermediary_proofs,
                // )
                // .await
                // {
                //     Err(_err) => continue,
                //     Ok(_) => {}
                // };
            }
        }
    }

    Ok(())
}

async fn process_first_level_task(
    con: &mut Connection,
    queue: &WorkQueue,
    queue_item: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    protocol: String,
    targets: &ValidatorBalanceVerificationAccumulatorTargets,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(queue_item.data[0..8].try_into().unwrap());

    println!("balance input index: {}", balance_input_index);
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
        fetch_validator_balance_accumulator_input(con, protocol, balance_input_index).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let mut pw = PartialWitness::new();

    targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = circuit_data.prove(pw)?;

    // let merkle_root_bits = proof.public_inputs[0..256]
    //     .iter()
    //     .map(|element| element.0)
    //     .collect_vec();
    //
    // let merkle_root = hex::encode(bits_to_bytes(merkle_root_bits.as_slice()));
    // println!("merkle_root: {}", merkle_root);

    match save_balance_accumulator_proof(con, proof, 0, balance_input_index).await {
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
            // TODO: Uncomment this
            // queue.complete(con, &queue_item).await?;
        }
    }

    Ok(())
}

fn get_first_level_targets() -> Result<Targets, anyhow::Error> {
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME, 0
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::FirstLevel(Some(
        ValidatorBalanceVerificationAccumulatorTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}

fn get_inner_level_targets(level: u64) -> Result<Targets> {
    let target_bytes = read_from_file(&format!(
        "{}/{}_{}.plonky2_targets",
        CIRCUIT_DIR, CIRCUIT_NAME, level
    ))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::InnerLevel(Some(
        BalanceInnerCircuitTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}

struct RecursiveCircuit {
    data: CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: Targets,
    lower_circuit_data: Option<CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
}

fn load_recursive_circuit_for_level(level: u64) -> Result<RecursiveCircuit> {
    let circuit_data = load_circuit_data(&format!("{}/{}_{}", CIRCUIT_DIR, CIRCUIT_NAME, level))?;

    let (lower_circuit_data, targets) = if level == 0 {
        (None, get_first_level_targets()?)
    } else {
        let lower_circuit_data = Some(load_circuit_data(&format!(
            "{}_{}",
            CIRCUIT_NAME,
            level - 1
        ))?);

        let targets = get_inner_level_targets(level)?;

        (lower_circuit_data, targets)
    };

    Ok(RecursiveCircuit {
        data: circuit_data,
        targets,
        lower_circuit_data,
    })
}
