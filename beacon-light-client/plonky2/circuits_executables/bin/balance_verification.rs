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
use circuits_executables::{
    crud::{
        delete_balance_verification_proof_dependencies, fetch_proofs_balances,
        fetch_validator_balance_input, load_circuit_data, read_from_file, save_balance_proof,
        BalanceProof,
    },
    provers::{handle_balance_inner_level_proof, SetPWValues},
    utils::parse_config_file,
    validator_commitment_constants::get_validator_commitment_constants,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};

use clap::{App, Arg};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

enum Targets {
    FirstLevel(Option<ValidatorBalanceVerificationTargets>),
    InnerLevel(Option<BalanceInnerCircuitTargets>),
}

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
            Arg::with_name("circuit_level")
                .short('l')
                .long("level")
                .value_name("LEVEL")
                .help("Sets the circuit level")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("stop_after")
            .long("stop-after")
            .value_name("Stop after")
            .help("Sets how many seconds to wait until the program stops if no new tasks are found in the queue")
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
            Arg::with_name("run_for_minutes")
                .long("run-for")
                .value_name("Run for X minutes")
                .takes_value(true)
                .default_value("infinity"),
        )
        .arg(
            Arg::with_name("preserve_intermediary_proofs")
            .long("preserve-intermediary-proofs")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let level = matches
        .value_of("circuit_level")
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let run_for_input = matches.value_of("run_for_minutes").unwrap();

    let time_to_run: Option<Duration> = match run_for_input {
        "infinity" => None,
        minutes => {
            let mins = minutes.parse::<u64>().expect("Failed to parse minutes");
            Some(Duration::from_secs(mins * 60))
        }
    };

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

    let preserve_intermediary_proofs = matches.get_flag("preserve_intermediary_proofs");

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!("Redis connection took: {:?}", elapsed);

    let start = Instant::now();

    let circuit_data = load_circuit_data(&level.to_string())?;

    let (inner_circuit_data, targets) = if level == 0 {
        (None, get_first_level_targets()?)
    } else {
        (
            Some(load_circuit_data(&format!("{}", level - 1))?),
            get_inner_level_targets(level)?,
        )
    };

    let elapsed = start.elapsed();

    println!("Circuit generation took: {:?}", elapsed);

    let queue = WorkQueue::new(KeyPrefix::new(format!(
        "{}:{}",
        get_validator_commitment_constants().balance_verification_queue,
        level
    )));

    println!("level {}", level);

    let start: Instant = Instant::now();

    process_queue(
        &mut con,
        &queue,
        &circuit_data,
        inner_circuit_data.as_ref(),
        &targets,
        level,
        start,
        time_to_run,
        stop_after,
        lease_for,
        preserve_intermediary_proofs,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    level: u64,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    preserve_intermediary_proofs: bool,
) -> Result<()> {
    while time_to_run.is_none() || start.elapsed() < time_to_run.unwrap() {
        let job = match queue
            .lease(
                con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(job) => job,
            None => {
                println!("No jobs left in queue");

                return Ok(());
            }
        };

        if job.data.is_empty() {
            println!("Skipping empty data job");
            queue.complete(con, &job).await?;

            continue;
        }

        println!("Processing job data: {:?}", job.data);

        match targets {
            Targets::FirstLevel(targets) => {
                match process_first_level_job(
                    con,
                    queue,
                    job,
                    circuit_data,
                    targets.as_ref().unwrap(),
                )
                .await
                {
                    Err(_err) => {
                        println!("Error processing first level job {:?}", _err);
                        continue;
                    }
                    Ok(_) => {}
                };
            }
            Targets::InnerLevel(inner_circuit_targets) => {
                match process_inner_level_job(
                    con,
                    queue,
                    job,
                    circuit_data,
                    inner_circuit_data.unwrap(),
                    inner_circuit_targets,
                    level,
                    preserve_intermediary_proofs,
                )
                .await
                {
                    Err(_err) => continue,
                    Ok(_) => {}
                };
            }
        }
    }

    Ok(())
}

async fn process_first_level_job(
    con: &mut Connection,
    queue: &WorkQueue,
    job: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &ValidatorBalanceVerificationTargets,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(job.data[0..8].try_into().unwrap());

    let start = Instant::now();
    let validator_balance_input = fetch_validator_balance_input(con, balance_input_index).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();

    let mut pw = PartialWitness::new();

    targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = circuit_data.prove(pw)?;

    match save_balance_proof(con, proof, 0, balance_input_index).await {
        Err(err) => {
            print!("Error: {}", err);
            thread::sleep(Duration::from_secs(5));
            return Err(err);
        }
        Ok(_) => {
            queue.complete(con, &job).await?;
        }
    }

    let elapsed = start.elapsed();

    println!("Proof generation took: {:?}", elapsed);

    Ok(())
}

async fn process_inner_level_job(
    con: &mut Connection,
    queue: &WorkQueue,
    job: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &Option<BalanceInnerCircuitTargets>,
    level: u64,
    preserve_intermediary_proofs: bool,
) -> Result<()> {
    let index = u64::from_be_bytes(job.data[0..8].try_into().unwrap());
    println!("Got index: {:?}", index);

    match fetch_proofs_balances::<BalanceProof>(con, level, index).await {
        Err(err) => {
            print!("Error: {}", err);
            return Err(err);
        }
        Ok(proofs) => {
            let start = Instant::now();

            let proof = handle_balance_inner_level_proof(
                proofs.0,
                proofs.1,
                &inner_circuit_data,
                &inner_circuit_targets.as_ref().unwrap(),
                &circuit_data,
            )?;

            match save_balance_proof(con, proof, level, index).await {
                Err(err) => {
                    print!("Error: {}", err);
                    thread::sleep(Duration::from_secs(5));
                    return Err(err);
                }
                Ok(_) => {
                    queue.complete(con, &job).await?;
                    if !preserve_intermediary_proofs {
                        // delete child nodes
                        delete_balance_verification_proof_dependencies(con, level, index).await?;
                    }
                }
            }

            let elapsed = start.elapsed();
            println!("Proof generation took: {:?}", elapsed);

            Ok(())
        }
    }
}

fn get_first_level_targets() -> Result<Targets, anyhow::Error> {
    let target_bytes = read_from_file(&format!("{}.plonky2_targets", 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::FirstLevel(Some(
        ValidatorBalanceVerificationTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}

fn get_inner_level_targets(level: u64) -> Result<Targets> {
    let target_bytes = read_from_file(&format!("{}.plonky2_targets", level))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::InnerLevel(Some(
        BalanceInnerCircuitTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}
