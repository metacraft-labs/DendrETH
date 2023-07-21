use std::{thread, time::Duration};

use crate::{
    validator::{Validator, VALIDATOR_REGISTRY_LIMIT}, validator_commitment_constants::get_validator_commitment_constants,
};
use anyhow::Result;
use plonky2::{plonk::{proof::ProofWithPublicInputs, config::PoseidonGoldilocksConfig}, field::goldilocks_field::GoldilocksField};
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<u64>,
    pub sha256_hash: Vec<u64>,
    pub proof: Vec<u8>,
}

pub async fn fetch_validator(con: &mut Connection, validator_index: usize) -> Result<Validator> {
    let json_str: String = con
        .get(format!(
            "{}:{}",
            get_validator_commitment_constants().validator_key,
            validator_index
        ))
        .await?;
    let validator: Validator = serde_json::from_str(&json_str)?;

    Ok(validator)
}

pub async fn save_validator_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let validator_proof = serde_json::to_string(&ValidatorProof {
        poseidon_hash: proof.public_inputs[0..4].iter().map(|x| x.0).collect(),
        sha256_hash: proof.public_inputs[4..260].iter().map(|x| x.0).collect(),
        proof: proof.to_bytes(),
        needs_change: false,
    })?;

    let _: () = con
        .set(
            format!(
                "{}:{}:{}",
                get_validator_commitment_constants().validator_proof_key,
                depth,
                index
            ),
            validator_proof,
        )
        .await?;

    Ok(())
}

pub async fn fetch_proof(
    con: &mut Connection,
    depth: usize,
    index: usize,
) -> Result<ValidatorProof> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result: Result<String, RedisError> = con
            .get(format!(
                "{}:{}:{}",
                get_validator_commitment_constants().validator_proof_key,
                depth,
                index
            ))
            .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = con
                .get(format!(
                    "{}:{}:{}",
                    get_validator_commitment_constants().validator_proof_key,
                    depth,
                    VALIDATOR_REGISTRY_LIMIT
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

pub async fn fetch_proofs(
    con: &mut Connection,
    indexes: &Vec<usize>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let proof1 = fetch_proof(con, indexes[0], indexes[1]).await?;
    let proof2 = fetch_proof(con, indexes[0], indexes[2]).await?;

    Ok((proof1.proof, proof2.proof))
}
