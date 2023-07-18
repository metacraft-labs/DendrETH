use std::{print, println, thread, time::Duration};

use anyhow::Result;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, RedisError};
use redis_work_queue::{KeyPrefix, WorkQueue};
use serde::{Deserialize, Serialize};

use circuits::{
    build_first_level_circuit::build_first_level_circuit,
    build_inner_level_circuit::{build_inner_circuit, InnerCircuitTargets},
    validator_commitment::ValidatorCommitment,
};
use futures_lite::future;

mod validator;

use validator::Validator;

const VALIDATOR_REGISTRY_LIMIT: usize = 1099511627776;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub proof: Vec<u8>,
}

async fn fetch_validator(con: &mut Connection, validator_index: usize) -> Result<Validator> {
    let json_str: String = con.get(format!("validator:{}", validator_index)).await?;
    let validator: Validator = serde_json::from_str(&json_str)?;

    Ok(validator)
}

async fn save_validator_proof(
    con: &mut Connection,
    proof: Vec<u8>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let validator_proof = serde_json::to_string(&ValidatorProof {
        proof,
        needs_change: false,
    })?;

    let _: () = con
        .set(
            format!("validator_proof:{}:{}", depth, index),
            validator_proof,
        )
        .await?;

    Ok(())
}

async fn fetch_proof(con: &mut Connection, depth: usize, index: usize) -> Result<ValidatorProof> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result: Result<String, RedisError> = con
            .get(format!("validator_proof:{}:{}", depth, index))
            .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = con
                .get(format!(
                    "validator_proof:{}:{}",
                    depth, VALIDATOR_REGISTRY_LIMIT
                ))
                .await;
        }

        let proof = serde_json::from_str::<ValidatorProof>(&proof_result?)?;

        if proof.needs_change {
            // Wait a bit and try again
            thread::sleep(Duration::from_secs(10));
            retries += 1;

            continue;
        }

        return Ok(proof);
    }
}

async fn fetch_proofs(con: &mut Connection, indexes: &Vec<usize>) -> Result<(Vec<u8>, Vec<u8>)> {
    let proof1 = fetch_proof(con, indexes[0], indexes[1]).await?;
    let proof2 = fetch_proof(con, indexes[0], indexes[2]).await?;

    Ok((proof1.proof, proof2.proof))
}

fn main() -> Result<()> {
    future::block_on(async_main())
}

async fn async_main() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_async_connection().await?;

    let queue = WorkQueue::new(KeyPrefix::new("validator_proofs".to_string()));

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

fn handle_inner_level_proof(
    proof1_bytes: Vec<u8>,
    proof2_bytes: Vec<u8>,
    inner_circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    inner_circuit_targets: &InnerCircuitTargets,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    is_zero: bool,
) -> Result<Vec<u8>> {
    let inner_proof1 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof1_bytes,
            &inner_circuit_data.common,
        )?;

    let inner_proof2 =
        ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            proof2_bytes,
            &inner_circuit_data.common,
        )?;

    let mut pw = PartialWitness::new();

    pw.set_proof_with_pis_target(&inner_circuit_targets.proof1, &inner_proof1);
    pw.set_proof_with_pis_target(&inner_circuit_targets.proof2, &inner_proof2);

    pw.set_cap_target(
        &inner_circuit_targets
            .verifier_circuit_target
            .constants_sigmas_cap,
        &inner_circuit_data.verifier_only.constants_sigmas_cap,
    );

    pw.set_hash_target(
        inner_circuit_targets.verifier_circuit_target.circuit_digest,
        inner_circuit_data.verifier_only.circuit_digest,
    );

    pw.set_bool_target(inner_circuit_targets.is_zero, is_zero);

    Ok(circuit_data.prove(pw)?.to_bytes())
}

fn handle_first_level_proof(
    validator: Validator,
    validator_commitment: &ValidatorCommitment,
    circuit_data: &CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<Vec<u8>> {
    let mut pw = PartialWitness::new();

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.pubkey,
        validator.pubkey,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawal_credentials,
        validator.withdrawal_credentials,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.effective_balance,
        validator.effective_balance,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.slashed,
        validator.slashed,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_eligibility_epoch,
        validator.activation_eligibility_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.activation_epoch,
        validator.activation_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.exit_epoch,
        validator.exit_epoch,
    );

    set_pw_values(
        &mut pw,
        &validator_commitment.validator.withdrawable_epoch,
        validator.withdrawable_epoch,
    );

    Ok(circuit_data.prove(pw)?.to_bytes())
}

fn set_pw_values(
    pw: &mut PartialWitness<GoldilocksField>,
    target: &[BoolTarget],
    source: Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}
