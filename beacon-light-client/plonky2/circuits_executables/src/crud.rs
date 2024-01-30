use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    validator::{
        bool_vec_as_int_vec, bool_vec_as_int_vec_nested, ValidatorShaInput,
        VALIDATOR_REGISTRY_LIMIT,
    },
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use anyhow::{ensure, Result};
use circuits::{
    build_commitment_mapper_first_level_circuit::CommitmentMapperProofExt,
    build_final_circuit::FinalCircuitProofExt,
    build_validator_balance_circuit::ValidatorBalanceProofExt,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
};
use num::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, JsonAsyncCommands};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<u64>,
    pub sha256_hash: Vec<u64>,
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof {
    pub needs_change: bool,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub range_total_value: BigUint,
    pub validators_commitment: Vec<u64>,
    pub balances_hash: Vec<u64>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub withdrawal_credentials: BigUint,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUint,
    pub proof: Vec<u8>,
}

pub fn biguint_to_str<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str_value = value.to_str_radix(10);
    serializer.serialize_str(&str_value)
}

pub fn parse_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let str_value = String::deserialize(deserializer)?;

    str_value
        .parse::<BigUint>()
        .map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FinalCircuitInput {
    #[serde(with = "bool_vec_as_int_vec")]
    pub state_root: Vec<bool>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUint,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub slot_branch: Vec<Vec<bool>>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub withdrawal_credentials: BigUint,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub balance_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub validators_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub validators_size_bits: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinalProof {
    pub needs_change: bool,
    pub state_root: Vec<u64>,
    pub withdrawal_credentials: BigUint,
    pub balance_sum: BigUint,
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
        VALIDATOR_COMMITMENT_CONSTANTS
            .validator_proof_key
            .to_owned()
    }
}

impl NeedsChange for BalanceProof {
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl KeyProvider for BalanceProof {
    fn get_key() -> String {
        VALIDATOR_COMMITMENT_CONSTANTS
            .balance_verification_proof_key
            .to_owned()
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
    index: u64,
) -> Result<ValidatorBalancesInput> {
    Ok(fetch_redis_json_object::<ValidatorBalancesInput>(
        con,
        format!(
            "{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_balance_input_key
                .to_owned(),
            index
        ),
    )
    .await?)
}

pub async fn fetch_final_layer_input(con: &mut Connection) -> Result<FinalCircuitInput> {
    let result: String = con
        .json_get(VALIDATOR_COMMITMENT_CONSTANTS.final_proof_input_key, "$")
        .await?;
    let result_vec = &serde_json::from_str::<Vec<FinalCircuitInput>>(&result)?;
    ensure!(!result_vec.is_empty(), "Could not fetch json object");
    Ok(result_vec[0].clone())
}

pub async fn save_balance_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: u64,
    index: u64,
) -> Result<()> {
    let balance_proof = BalanceProof {
        needs_change: false,
        range_total_value: proof.get_range_total_value(),
        balances_hash: proof.get_range_balances_root().to_vec(),
        withdrawal_credentials: proof.get_withdrawal_credentials(),
        validators_commitment: proof.get_range_validator_commitment().to_vec(),
        current_epoch: proof.get_current_epoch(),
        proof: proof.to_bytes(),
    };

    con.json_set(
        format!(
            "{}:{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .balance_verification_proof_key
                .to_owned(),
            level,
            index
        ),
        "$",
        &balance_proof,
    )
    .await?;

    Ok(())
}

pub async fn save_final_proof(
    con: &mut Connection,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<()> {
    let final_proof = FinalProof {
        needs_change: false,
        state_root: proof.get_final_circuit_state_root().to_vec(),
        withdrawal_credentials: proof.get_final_circuit_withdrawal_credentials(),
        balance_sum: proof.get_final_circuit_balance_sum(),
        proof: proof.to_bytes(),
    };

    con.json_set(
        VALIDATOR_COMMITMENT_CONSTANTS
            .final_layer_proof_key
            .to_owned(),
        "$",
        &final_proof,
    )
    .await?;

    Ok(())
}

pub async fn delete_balance_verification_proof_dependencies(
    con: &mut Connection,
    level: u64,
    index: u64,
) -> Result<()> {
    let prefix = format!(
        "{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS
            .balance_verification_proof_key
            .to_owned(),
        level - 1
    );

    con.del(format!("{}:{}", prefix, index * 2,)).await?;
    con.del(format!("{}:{}", prefix, index * 2 + 1,)).await?;

    let result: Vec<String> = con.keys(format!("{}:*", prefix)).await?;
    if result.len() == 1 {
        con.del(format!("{}:{}", prefix, VALIDATOR_REGISTRY_LIMIT))
            .await?;
    }

    Ok(())
}

pub async fn get_latest_epoch(con: &mut Connection, key: &String, epoch: u64) -> Result<String> {
    let result: Vec<String> = con
        .zrevrangebyscore_limit(
            format!(
                "{}:{}",
                key,
                VALIDATOR_COMMITMENT_CONSTANTS.epoch_lookup_key.to_owned(),
            ),
            epoch,
            0,
            0,
            1,
        )
        .await?;

    ensure!(!result.is_empty(), "Could not find data for epoch");
    Ok(result[0].clone())
}

pub async fn fetch_validator(
    con: &mut Connection,
    validator_index: u64,
    epoch: u64,
) -> Result<ValidatorShaInput> {
    let key = format!(
        "{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS.validator_key.to_owned(),
        validator_index
    );

    let latest_epoch = get_latest_epoch(con, &key, epoch).await?;
    Ok(
        fetch_redis_json_object::<ValidatorShaInput>(con, format!("{}:{}", key, latest_epoch))
            .await?,
    )
}

pub async fn save_zero_validator_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: u64,
) -> Result<()> {
    let validator_proof = ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        proof: proof.to_bytes(),
        needs_change: false,
    };

    con.json_set(
        format!(
            "{}:zeroes:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_proof_key
                .to_owned(),
            depth,
        ),
        "$",
        &validator_proof,
    )
    .await?;

    Ok(())
}

pub async fn save_validator_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    gindex: u64,
    epoch: u64,
) -> Result<()> {
    let validator_proof = ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        proof: proof.to_bytes(),
        needs_change: false,
    };

    con.json_set(
        format!(
            "{}:{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_proof_key
                .to_owned(),
            gindex,
            epoch
        ),
        "$",
        &validator_proof,
    )
    .await?;

    Ok(())
}

pub async fn fetch_zero_proof<T: NeedsChange + KeyProvider + DeserializeOwned + Clone>(
    con: &mut Connection,
    depth: u64,
) -> Result<T> {
    let mut retries = 0;
    loop {
        let proof =
            fetch_redis_json_object::<T>(con, format!("{}:zeroes:{}", T::get_key(), depth)).await?;

        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        if proof.needs_change() {
            // Wait a bit and try again
            thread::sleep(Duration::from_secs(10));
            retries += 1;
            continue;
        }
        return Ok(proof);
    }
}

pub async fn fetch_redis_json_object<T: DeserializeOwned + Clone>(
    con: &mut Connection,
    key: String,
) -> Result<T> {
    let result: String = con.json_get(key, "$").await?;
    let result_vec = &serde_json::from_str::<Vec<T>>(&result)?;
    ensure!(!result_vec.is_empty(), "Could not fetch json object");
    Ok(result_vec[0].clone())
}

pub async fn fetch_proof<T: NeedsChange + KeyProvider + DeserializeOwned + Clone>(
    con: &mut Connection,
    gindex: u64,
    epoch: u64,
) -> Result<T> {
    let key = format!("{}:{}", T::get_key(), gindex);
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let latest_epoch_result = get_latest_epoch(con, &key, epoch).await;

        let proof = match latest_epoch_result {
            Ok(latest_epoch) => {
                let proof_result = fetch_redis_json_object::<T>(
                    con,
                    format!("{}:{}:{}", T::get_key(), gindex, latest_epoch),
                )
                .await;

                match proof_result {
                    Ok(res) => res,
                    Err(_) => fetch_zero_proof::<T>(con, get_depth_for_gindex(gindex)).await?,
                }
            }
            Err(_) => fetch_zero_proof::<T>(con, get_depth_for_gindex(gindex)).await?,
        };

        if proof.needs_change() {
            // Wait a bit and try again
            thread::sleep(Duration::from_secs(10));
            retries += 1;

            continue;
        }

        return Ok(proof);
    }
}

pub async fn fetch_proofs<
    T: NeedsChange + KeyProvider + ProofProvider + DeserializeOwned + Clone,
>(
    con: &mut Connection,
    gindex: u64,
    epoch: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let left_child_gindex = gindex * 2 + 1;
    let right_child_gindex = gindex * 2 + 2;

    let proof1 = fetch_proof::<T>(con, left_child_gindex, epoch).await?;
    let proof2 = fetch_proof::<T>(con, right_child_gindex, epoch).await?;

    Ok((proof1.get_proof(), proof2.get_proof()))
}

// @TODO: Rename this later
pub async fn fetch_proof_balances<T: NeedsChange + KeyProvider + DeserializeOwned + Clone>(
    con: &mut Connection,
    level: u64,
    index: u64,
) -> Result<T> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result =
            fetch_redis_json_object::<T>(con, format!("{}:{}:{}", T::get_key(), level, index))
                .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = fetch_redis_json_object::<T>(
                con,
                format!("{}:{}:{}", T::get_key(), level, VALIDATOR_REGISTRY_LIMIT),
            )
            .await;
        }

        let proof = proof_result?;

        if proof.needs_change() {
            // Wait a bit and try again
            thread::sleep(Duration::from_secs(10));
            retries += 1;

            continue;
        }

        return Ok(proof);
    }
}

// @TODO: Rename this later
pub async fn fetch_proofs_balances<
    T: NeedsChange + KeyProvider + ProofProvider + DeserializeOwned + Clone,
>(
    con: &mut Connection,
    level: u64,
    index: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (left_child_index, right_child_index) = (index * 2, index * 2 + 1);

    let proof1 = fetch_proof_balances::<T>(con, level - 1, left_child_index).await?;
    let proof2 = fetch_proof_balances::<T>(con, level - 1, right_child_index).await?;

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

pub fn load_circuit_data(
    file_name: &str,
) -> Result<CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
    let gate_serializer = DendrETHGateSerializer;
    let generator_serializer = DendrETHGeneratorSerializer {
        _phantom: PhantomData::<PoseidonGoldilocksConfig>,
    };

    let circuit_data_bytes = read_from_file(&format!("{}.plonky2_circuit", file_name))?;

    Ok(
        CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            &circuit_data_bytes,
            &gate_serializer,
            &generator_serializer,
        )
        .unwrap(),
    )
}

pub fn get_depth_for_gindex(gindex: u64) -> u64 {
    (gindex + 1).ilog2() as u64
}
