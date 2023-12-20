use anyhow::Result;
use circuits::{
    build_commitment_mapper_inner_level_circuit::CommitmentMapperInnerCircuitTargets,
    targets_serialization::ReadTargets, validator_commitment_mapper::ValidatorCommitmentTargets,
};
use circuits_executables::{
    crud::{
        fetch_proofs, fetch_validator, load_circuit_data, read_from_file, save_validator_proof,
        ValidatorProof,
    },
    provers::{handle_commitment_mapper_inner_level_proof, SetPWValues},
    validator::VALIDATOR_REGISTRY_LIMIT,
    validator_commitment_constants,
};
use clap::{App, Arg};
use futures_lite::future;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, print, println, thread, time::Duration};

use validator_commitment_constants::get_validator_commitment_constants;

use jemallocator::Jemalloc;

use serde_binary::binary_stream;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

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
    .get_matches();

    let redis_connection = matches.value_of("redis_connection").unwrap();

    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_async_connection().await?;

    println!("Connected to redis");

    let queue = WorkQueue::new(KeyPrefix::new(
        get_validator_commitment_constants().validator_proofs_queue,
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
            println!("Waiting for job...");
        }

        let job = match queue
            .lease(
                &mut con,
                Some(Duration::from_secs(stop_after)),
                Duration::from_secs(lease_for),
            )
            .await?
        {
            Some(job) => job,
            None => continue,
        };

        if job.data.len() == 8 {
            let validator_index = u64::from_be_bytes(job.data[0..8].try_into().unwrap()) as usize;

            if !mock {
                println!("Validator index {}", validator_index);
            } else if validator_index % 10000 == 0 {
                println!("Validator index {}", validator_index);
            }

            match fetch_validator(&mut con, validator_index).await {
                Err(err) => {
                    print!("Error: {}", err);
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
                Ok(validator) => {
                    let mut pw = PartialWitness::new();

                    validator_commitment
                        .validator
                        .set_pw_values(&mut pw, &validator);

                    let proof = if mock {
                        serde_binary::from_slice(proof_mock_binary, binary_stream::Endian::Big)
                            .unwrap()
                    } else {
                        first_level_circuit_data.prove(pw)?
                    };

                    match save_validator_proof(&mut con, proof, 0, validator_index).await {
                        Err(err) => {
                            print!("Error: {}", err);
                            thread::sleep(Duration::from_secs(10));
                            continue;
                        }
                        Ok(_) => {
                            queue.complete(&mut con, &job).await?;
                        }
                    }
                }
            }
        } else if job.data.len() == 24 {
            let proof_indexes = job
                .data
                .chunks(8)
                .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()) as usize)
                .collect::<Vec<usize>>();

            if !mock {
                println!("Got indexes: {:?}", proof_indexes);
            } else if proof_indexes[1] % 2048 == 0 {
                println!("Got indexes: {:?}", proof_indexes);
            }

            match fetch_proofs::<ValidatorProof>(&mut con, &proof_indexes).await {
                Err(err) => {
                    print!("Error: {}", err);
                    continue;
                }
                Ok(proofs) => {
                    let inner_circuit_data = if proof_indexes[0] > 0 {
                        &inner_circuits[proof_indexes[0] - 1].1
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
                            &inner_circuits[proof_indexes[0]].0,
                            &inner_circuits[proof_indexes[0]].1,
                            proof_indexes[2] == VALIDATOR_REGISTRY_LIMIT && proof_indexes[0] == 0,
                        )?
                    };

                    match save_validator_proof(
                        &mut con,
                        proof,
                        proof_indexes[0] + 1,
                        proof_indexes[1],
                    )
                    .await
                    {
                        Err(err) => {
                            print!("Error: {}", err);
                            thread::sleep(Duration::from_secs(10));
                            continue;
                        }
                        Ok(_) => {
                            queue.complete(&mut con, &job).await?;
                        }
                    }
                }
            };
        } else {
            println!("Invalid job data");
            println!("This is bug from somewhere");

            queue.complete(&mut con, &job).await?;
        }
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
