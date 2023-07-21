use std::{print, println, thread, time::Duration};
use anyhow::Result;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig,
    },
};
use redis_work_queue::{KeyPrefix, WorkQueue};
use circuits::{
    build_first_level_circuit::build_first_level_circuit,
    build_inner_level_circuit::{build_inner_circuit, InnerCircuitTargets},
};
use futures_lite::future;

mod validator;
mod validator_commitment_constants;
mod provers;
mod crud;

use validator_commitment_constants::get_validator_commitment_constants;
use provers::handle_first_level_proof;

use crate::{provers::handle_inner_level_proof, crud::{fetch_validator, save_validator_proof, fetch_proofs}, validator::VALIDATOR_REGISTRY_LIMIT};


fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_async_connection().await?;

    let queue = WorkQueue::new(KeyPrefix::new(
        get_validator_commitment_constants().validator_proofs_queue,
    ));

    let (validator_commitment, first_level_circuit_data) = build_first_level_circuit();

    let mut inner_circuits: Vec<(
        InnerCircuitTargets,
        CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    )> = Vec::with_capacity(40);

    inner_circuits.push(build_inner_circuit(&first_level_circuit_data));

    for i in 1..40 {
        inner_circuits.push(build_inner_circuit(&inner_circuits[i - 1].1));
    }

    loop {
        println!("Waiting for job...");

        let job = match queue
            .lease(&mut con, Option::None, Duration::from_secs(600))
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
                    let proof = handle_first_level_proof(
                        validator,
                        &validator_commitment,
                        &first_level_circuit_data,
                    )?;

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

            match fetch_proofs(&mut con, &proof_indexes).await {
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

                    let proof = handle_inner_level_proof(
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
