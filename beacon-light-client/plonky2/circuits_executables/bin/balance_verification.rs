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
        fetch_proofs, fetch_validator_balance_input, load_circuit_data, read_from_file,
        save_balance_proof, BalanceProof, FileStorage, ProofStorage,
    },
    provers::{handle_balance_inner_level_proof, SetPWValues},
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::get_validator_commitment_constants,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
    util::serialization::Buffer,
};

use clap::{App, Arg};

use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

use jemallocator::Jemalloc;

use serde_binary::binary_stream;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
static INNER_PROOF_MOCK_BINARY: &[u8; 214532] =
    include_bytes!("../mock_data/inner_proof_verified.mock");
static PROOF_MOCK_BINARY: &[u8; 204016] = include_bytes!("../mock_data/proof_verified.mock");

enum Targets {
    FirstLevel(Option<ValidatorBalanceVerificationTargets>),
    InnerLevel(Option<BalanceInnerCircuitTargets>),
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let matches = App::new("")
        .arg(
            Arg::with_name("redis_connection")
                .short('r')
                .long("redis")
                .value_name("Redis Connection")
                .help("Sets a custom Redis connection")
                .takes_value(true)
                .default_value("redis://127.0.0.1:6379/"),
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
            Arg::with_name("run_for_minutes")
                .long("run-for")
                .value_name("Run for X minutes")
                .takes_value(true)
                .default_value("infinity"),
        )
        .arg(
            Arg::with_name("mock")
            .long("mock")
            .help("Sets mock mode")
            .takes_value(false)
            .default_value("false")
        )
        .get_matches();

    let level = matches
        .value_of("circuit_level")
        .unwrap()
        .parse::<usize>()
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

    let mock = matches.value_of("mock").unwrap().parse::<bool>().unwrap();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;

    let mut con = client.get_async_connection().await?;

    let mut proof_storage = FileStorage::new();

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
        &mut proof_storage,
        &queue,
        &circuit_data,
        inner_circuit_data.as_ref(),
        &targets,
        level,
        start,
        time_to_run,
        stop_after,
        lease_for,
        mock,
    )
    .await
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    level: usize,
    start: Instant,
    time_to_run: Option<Duration>,
    stop_after: u64,
    lease_for: u64,
    mock: bool,
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
                    proof_storage,
                    queue,
                    job,
                    circuit_data,
                    targets.as_ref().unwrap(),
                    mock,
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
                    proof_storage,
                    queue,
                    job,
                    circuit_data,
                    inner_circuit_data.unwrap(),
                    inner_circuit_targets,
                    level,
                    mock,
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
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    job: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    targets: &ValidatorBalanceVerificationTargets,
    mock: bool,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(job.data[0..8].try_into().unwrap()) as usize;

    let start = Instant::now();

    let validator_balance_input_mock: ValidatorBalancesInput = serde_json::from_str(r#"{"validators":[{"pubkey":"28155455410776443498482448245529435809359956175169908048702124937172512699631467778238641261986684575799579397993811","withdrawalCredentials":"382737961640003689533910812141398037732875097701826973858731813319968970850","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"26979846440152886011310362764920484765984271438301878724818610563677684689774592103308019846399354324216936243366261","withdrawalCredentials":"334836033201229227961245502702526419573885144848257583169567123681803257233","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"27801740282514395063227256283476233197453489056913878482578327955357746398789954319627803579173623839636086204205312","withdrawalCredentials":"452312848583266388373324161311539720301441557426889082580326305166421027472","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"21527346288222609476208495591203643471148391088705253248992136372752069313676131849343273583439774745199246011648576","withdrawalCredentials":"367671889103706031716807112321536209552784367704779136855912917662270427673","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"22320248583573513380499645876659337063375986179485491270050641054377962534820064449058321021365382800967547431872178","withdrawalCredentials":"452312848583266388373324160271404058109911226905144332655132243525891128419","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"21619640184761671406845477913809792923864724567879069222980681511978293118339791878883269161616622145838709233406408","withdrawalCredentials":"13798419507846026553622359324325711518785000279005931307706727393517509549","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"28315493868134351375148990743790149058304089158549908131347185017299284710487426690336332785304639119792063400013028","withdrawalCredentials":"3043153795794382687251851273790182871169017274083796406622225121057728559","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"},{"pubkey":"25138136466776665720292214356534543105788238163001562454030604153595944200262426495626688131184611875744151551092852","withdrawalCredentials":"452312848583266388373324160200794100136802167846724101891059603667205939461","effectiveBalance":"32000000000","slashed":0,"activationEligibilityEpoch":"0","activationEpoch":"0","exitEpoch":"18446744073709551615","withdrawableEpoch":"18446744073709551615"}],"balances":[[0,1,1,0,0,1,0,0,0,1,0,0,0,0,0,1,0,0,1,0,1,1,0,1,1,0,0,0,1,1,1,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,1,0,0,0,1,1,0,0,1,1,0,1,1,0,1,1,0,1,0,0,0,1,1,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,1,0,1,0,0,0,1,0,0,1,0,1,1,1,0,1,1,0,1,1,1,0,0,1,1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,1,0,0,1,0,0,0,0,0,1,0,1,0,0,0,1,0,0,1,0,0,0,1,1,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],[0,0,0,0,1,0,1,0,1,0,0,0,1,1,1,1,1,0,1,1,1,0,1,0,0,1,1,1,0,0,1,1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,0,0,0,1,0,0,0,0,0,1,1,1,1,0,1,0,0,0,0,1,0,0,1,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,0,0,1,1,1,0,0,0,1,1,1,1,1,1,0,1,0,1,0,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,1,0,1,0,0,0,1,1,0,1,1,1,0,1,1,0,1,1,1,0,0,1,1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]],"withdrawalCredentials":"452312848583266388373324160315533450859862645952655026790676503269398455083","currentEpoch":"217293","validatorIsZero":[0,0,0,0,0,0,0,0]}"#).unwrap();
    let validator_balance_input = if mock {
        validator_balance_input_mock
    } else {
        fetch_validator_balance_input(con, balance_input_index).await?
    };

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();

    let mut pw = PartialWitness::new();

    targets.set_pw_values(&mut pw, &validator_balance_input);

    let proof = if mock {
        let proof_mock: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> =
            serde_binary::from_slice(PROOF_MOCK_BINARY, binary_stream::Endian::Big).unwrap();
        proof_mock
    } else {
        circuit_data.prove(pw)?
    };

    match save_balance_proof(con, proof_storage, proof, 0, balance_input_index).await {
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
    proof_storage: &mut dyn ProofStorage,
    queue: &WorkQueue,
    job: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &Option<BalanceInnerCircuitTargets>,
    level: usize,
    mock: bool,
) -> Result<()> {
    let proof_indexes = job
        .data
        .chunks(8)
        .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()) as usize)
        .collect::<Vec<usize>>();

    println!("Got indexes: {:?}", proof_indexes);

    match fetch_proofs::<BalanceProof>(con, proof_storage, &proof_indexes).await {
        Err(err) => {
            print!("Error: {}", err);
            return Err(err);
        }
        Ok(proofs) => {
            let start = Instant::now();

            let proof = if mock {
                let inner_proof_mock: ProofWithPublicInputs<
                    GoldilocksField,
                    PoseidonGoldilocksConfig,
                    2,
                > = serde_binary::from_slice(INNER_PROOF_MOCK_BINARY, binary_stream::Endian::Big)
                    .unwrap();
                inner_proof_mock
            } else {
                handle_balance_inner_level_proof(
                    proofs.0,
                    proofs.1,
                    &inner_circuit_data,
                    &inner_circuit_targets.as_ref().unwrap(),
                    &circuit_data,
                )?
            };

            match save_balance_proof(con, proof_storage, proof, level, proof_indexes[1]).await {
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
    }
}

fn get_first_level_targets() -> Result<Targets, anyhow::Error> {
    let target_bytes = read_from_file(&format!("{}.plonky2_targets", 0))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::FirstLevel(Some(
        ValidatorBalanceVerificationTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}

fn get_inner_level_targets(level: usize) -> Result<Targets> {
    let target_bytes = read_from_file(&format!("{}.plonky2_targets", level))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(Targets::InnerLevel(Some(
        BalanceInnerCircuitTargets::read_targets(&mut target_buffer).unwrap(),
    )))
}
