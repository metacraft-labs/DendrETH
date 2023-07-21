use std::{
    marker::PhantomData,
    println, thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use circuits::{
    build_inner_level_circuit::InnerCircuitTargets,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    validator_hash_tree_root_poseidon::ValidatorPoseidon,
};
use circuits_executables::{
    crud::{
        fetch_proofs, fetch_validator_balance_input, read_from_file, save_balance_proof,
        BalanceProof,
    },
    provers::{
        handle_inner_level_proof, set_boolean_pw_values, set_pw_values, set_validator_pw_values,
    },
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants::get_validator_commitment_constants,
};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::{BoolTarget, Target},
        witness::PartialWitness,
    },
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::{Buffer, Read},
};

use clap::{App, Arg};
use jemallocator::Jemalloc;
use redis::aio::Connection;
use redis_work_queue::{Item, KeyPrefix, WorkQueue};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

enum Targets {
    FirstLevel(
        Option<Vec<Vec<BoolTarget>>>,
        Option<Vec<ValidatorPoseidon>>,
        Option<Vec<Target>>,
    ),
    InnerLevel(Option<InnerCircuitTargets>),
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
        .get_matches();

    let level = matches.value_of("circuit_level").unwrap().parse::<usize>().unwrap();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let start = Instant::now();
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    let elapsed = start.elapsed();

    println!("Redis connection took: {:?}", elapsed);

    let start = Instant::now();
    let gate_serializer = DendrETHGateSerializer;
    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_data = load_circuit_data(&gate_serializer, &generator_serializer, level)?;

    let (inner_circuit_data, targets) = if level == 0 {
        (None, get_first_level_targets()?)
    } else {
        (
            Some(load_circuit_data(
                &gate_serializer,
                &generator_serializer,
                level - 1,
            )?),
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

    process_queue(
        &mut con,
        &queue,
        &circuit_data,
        inner_circuit_data.as_ref(),
        &targets,
        level,
    )
    .await
}

fn load_circuit_data(
    gate_serializer: &DendrETHGateSerializer,
    generator_serializer: &DendrETHGeneratorSerializer<PoseidonGoldilocksConfig, 2>,
    level: usize,
) -> Result<CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    let circuit_data_bytes = read_from_file(&format!("{}.plonky2_circuit", level))?;

    Ok(
        CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            &circuit_data_bytes,
            gate_serializer,
            generator_serializer,
        )
        .unwrap(),
    )
}

async fn process_queue(
    con: &mut redis::aio::Connection,
    queue: &WorkQueue,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_data: Option<&CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>>,
    targets: &Targets,
    level: usize,
) -> Result<()> {
    loop {
        let job = match queue
            .lease(con, Option::None, Duration::from_secs(30))
            .await?
        {
            Some(job) => job,
            None => continue,
        };

        if job.data.is_empty() {
            println!("Skipping empty data job");
            queue.complete(con, &job).await?;
            continue;
        }

        println!("Processing job data: {:?}", job.data);

        match targets {
            Targets::FirstLevel(
                balances_targets,
                validators_targets,
                withdrawal_credentials_targets,
            ) => {
                match process_first_level_job(
                    con,
                    queue,
                    job,
                    circuit_data,
                    &balances_targets,
                    &validators_targets,
                    &withdrawal_credentials_targets,
                )
                .await
                {
                    Err(_err) => continue,
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
                )
                .await
                {
                    Err(_err) => continue,
                    Ok(_) => {}
                };
            }
        }
    }
}

async fn process_first_level_job(
    con: &mut Connection,
    queue: &WorkQueue,
    job: Item,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    balances_targets: &Option<Vec<Vec<BoolTarget>>>,
    validators_targets: &Option<Vec<ValidatorPoseidon>>,
    withdrawal_credentials_targets: &Option<Vec<Target>>,
) -> Result<()> {
    let balance_input_index = u64::from_be_bytes(job.data[0..8].try_into().unwrap()) as usize;

    let start = Instant::now();
    let validator_balance_input = fetch_validator_balance_input(con, balance_input_index).await?;

    let elapsed = start.elapsed();

    println!("Fetching validator balance input took: {:?}", elapsed);

    let start = Instant::now();

    let mut pw = PartialWitness::new();

    for i in 0..validator_balance_input.balances.len() {
        set_boolean_pw_values(
            &mut pw,
            &balances_targets.as_ref().unwrap()[i],
            validator_balance_input.balances[i].clone(),
        );
    }

    for i in 0..validator_balance_input.validators.len() {
        set_validator_pw_values(
            &mut pw,
            &validators_targets.as_ref().unwrap()[i],
            &validator_balance_input.validators[i],
        );
    }

    set_pw_values(
        &mut pw,
        &withdrawal_credentials_targets.as_ref().unwrap(),
        validator_balance_input.withdrawal_credentials,
    );

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
    inner_circuit_targets: &Option<InnerCircuitTargets>,
    level: usize,
) -> Result<()> {
    let proof_indexes = job
        .data
        .chunks(8)
        .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()) as usize)
        .collect::<Vec<usize>>();

    println!("Got indexes: {:?}", proof_indexes);

    match fetch_proofs::<BalanceProof>(con, &proof_indexes).await {
        Err(err) => {
            print!("Error: {}", err);
            return Err(err);
        }
        Ok(proofs) => {
            let start = Instant::now();

            let proof = handle_inner_level_proof(
                proofs.0,
                proofs.1,
                &inner_circuit_data,
                &inner_circuit_targets.as_ref().unwrap(),
                &circuit_data,
                proof_indexes[2] == VALIDATOR_REGISTRY_LIMIT && proof_indexes[0] == 0,
            )?;

            match save_balance_proof(con, proof, level, proof_indexes[1]).await {
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
    let mut balances_targets = Vec::<Vec<BoolTarget>>::new();
    let mut validators_targets = Vec::<ValidatorPoseidon>::new();
    for _ in 0..2 {
        balances_targets.push(target_buffer.read_target_bool_vec().unwrap());
    }
    for _ in 0..8 {
        validators_targets.push(ValidatorPoseidon {
            pubkey: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            withdrawal_credentials: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            effective_balance: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            slashed: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            activation_eligibility_epoch: target_buffer
                .read_target_vec()
                .unwrap()
                .try_into()
                .unwrap(),
            activation_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            exit_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
            withdrawable_epoch: target_buffer.read_target_vec().unwrap().try_into().unwrap(),
        });
    }
    let withdrawal_credentials = target_buffer.read_target_vec().unwrap();

    Ok(Targets::FirstLevel(
        Some(balances_targets),
        Some(validators_targets),
        Some(withdrawal_credentials),
    ))
}

fn get_inner_level_targets(level: usize) -> Result<Targets> {
    let target_bytes = read_from_file(&format!("{}.plonky2_targets", level))?;
    let mut target_buffer = Buffer::new(&target_bytes);

    let proof1 = target_buffer
        .read_target_proof_with_public_inputs()
        .unwrap();
    let proof2 = target_buffer
        .read_target_proof_with_public_inputs()
        .unwrap();
    let verifier_circuit_target = target_buffer.read_target_verifier_circuit().unwrap();
    let is_zero = target_buffer.read_target_bool().unwrap();

    Ok(Targets::InnerLevel(Some(InnerCircuitTargets {
        proof1: proof1,
        proof2: proof2,
        verifier_circuit_target: verifier_circuit_target,
        is_zero: is_zero,
    })))
}
