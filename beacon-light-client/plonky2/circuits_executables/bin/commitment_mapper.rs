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
use futures_lite::future;
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field64},
    iop::witness::PartialWitness,
    plonk::{circuit_data::CircuitData, config::PoseidonGoldilocksConfig},
    util::serialization::Buffer,
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use std::{format, print, println, thread, time::Duration};

use validator_commitment_constants::get_validator_commitment_constants;

use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_async_connection().await?;

    let queue = WorkQueue::new(KeyPrefix::new(
        get_validator_commitment_constants().validator_proofs_queue,
    ));

    let first_level_circuit_data = load_circuit_data("commitment_mapper_0")?;
    let validator_commitment = get_first_level_targets()?;

    let mut inner_circuits: Vec<(
        CommitmentMapperInnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::new();

    for i in 1..42 {
        inner_circuits.push((
            get_inner_targets(i)?,
            load_circuit_data(&format!("commitment_mapper_{}", i))?,
        ));
    }

    loop {
        println!("Waiting for job...");

        let job = match queue
            .lease(&mut con, Option::None, Duration::from_secs(20))
            .await?
        {
            Some(job) => job,
            None => continue,
        };

        println!("Got job: {:?}", job.data);

        if job.data.len() == 8 {
            let validator_index = u64::from_be_bytes(job.data[0..8].try_into().unwrap()) as usize;

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

                    let proof = first_level_circuit_data.prove(pw)?;

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

            println!("Got indexes: {:?}", proof_indexes);

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

                    let proof = handle_commitment_mapper_inner_level_proof(
                        proofs.0,
                        proofs.1,
                        inner_circuit_data,
                        &inner_circuits[proof_indexes[0]].0,
                        &inner_circuits[proof_indexes[0]].1,
                        proof_indexes[2] == VALIDATOR_REGISTRY_LIMIT && proof_indexes[0] == 0,
                    )?;

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
