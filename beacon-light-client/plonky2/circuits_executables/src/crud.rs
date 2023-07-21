use std::{thread, time::Duration, fs};

use crate::{
    validator::{Validator, VALIDATOR_REGISTRY_LIMIT},
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::get_validator_commitment_constants,
};
use anyhow::Result;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs},
};
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<u64>,
    pub sha256_hash: Vec<u64>,
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof {
    pub needs_change: bool,
    pub range_total_value: u64,
    pub validators_commitment: Vec<u64>,
    pub balances_hash: Vec<u64>,
    pub withdrawal_credentials: Vec<u64>,
    pub proof: Vec<u8>,
}

pub trait NeedsChange {
    fn needs_change(&self) -> bool;
}

pub trait KeyProvider {
    fn get_key() -> String;
}

pub trait ProofProvider {
    fn get_proof(&self) -> Vec<u8>;
}

impl NeedsChange for ValidatorProof {
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl KeyProvider for ValidatorProof {
    fn get_key() -> String {
        get_validator_commitment_constants().validator_proof_key
    }
}

impl NeedsChange for BalanceProof {
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl KeyProvider for BalanceProof {
    fn get_key() -> String {
        get_validator_commitment_constants().balance_verification_proof_key
    }
}

impl ProofProvider for ValidatorProof {
    fn get_proof(&self) -> Vec<u8> {
        self.proof.clone()
    }
}

impl ProofProvider for BalanceProof {
    fn get_proof(&self) -> Vec<u8> {
        self.proof.clone()
    }
}

pub async fn fetch_validator_balance_input(
    con: &mut Connection,
    index: usize,
) -> Result<ValidatorBalancesInput> {
    let json_str: String = con
        .get(format!(
            "{}:{}",
            get_validator_commitment_constants().validator_balance_input_key,
            index
        ))
        .await?;

    let validator_balance_input: ValidatorBalancesInput = serde_json::from_str(&json_str)?;

    Ok(validator_balance_input)
}

pub async fn save_balance_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let balance_proof = serde_json::to_string(&BalanceProof {
        needs_change: false,
        range_total_value: proof.public_inputs[0].0,
        balances_hash: proof.public_inputs[1..257].iter().map(|x| x.0).collect(),
        withdrawal_credentials: proof.public_inputs[257..262].iter().map(|x| x.0).collect(),
        validators_commitment: proof.public_inputs[262..266].iter().map(|x| x.0).collect(),
        proof: proof.to_bytes(),
    })?;

    let _: () = con
        .set(
            format!(
                "{}:{}:{}",
                get_validator_commitment_constants().balance_verification_proof_key,
                depth,
                index
            ),
            balance_proof,
        )
        .await?;

    Ok(())
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

pub async fn fetch_proof<T: NeedsChange + KeyProvider + DeserializeOwned>(
    con: &mut Connection,
    depth: usize,
    index: usize,
) -> Result<T> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result: Result<String, RedisError> = con
            .get(format!("{}:{}:{}", T::get_key(), depth, index))
            .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = con
                .get(format!(
                    "{}:{}:{}",
                    T::get_key(),
                    depth,
                    VALIDATOR_REGISTRY_LIMIT
                ))
                .await;
        }

        let proof = serde_json::from_str::<T>(&proof_result?)?;

        if proof.needs_change() {
            // Wait a bit and try again
            thread::sleep(Duration::from_secs(10));
            retries += 1;

            continue;
        }

        return Ok(proof);
    }
}

pub async fn fetch_proofs<T: NeedsChange + KeyProvider + ProofProvider + DeserializeOwned>(
    con: &mut Connection,
    indexes: &Vec<usize>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let proof1 = fetch_proof::<T>(con, indexes[0], indexes[1]).await?;
    let proof2 = fetch_proof::<T>(con, indexes[0], indexes[2]).await?;

    Ok((proof1.get_proof(), proof2.get_proof()))
}

pub fn read_from_file(file_path: &str) -> Result<Vec<u8>> {
    let data = fs::read(file_path)?;
    Ok(data)
}

pub fn write_to_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::write(file_path, data)?;
    Ok(())
}
