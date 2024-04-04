use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    utils::get_depth_for_gindex,
    validator::{
        bool_vec_as_int_vec, bool_vec_as_int_vec_nested, ValidatorShaInput,
        VALIDATOR_REGISTRY_LIMIT,
    },
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use anyhow::{ensure, Result};
use async_trait::async_trait;

use circuits::{
    build_commitment_mapper_first_level_circuit::CommitmentMapperProofExt,
    build_validator_balance_circuit::ValidatorBalanceProofExt,
    generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    utils::hash_bytes,
};
use num::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use super::proof_storage::proof_storage::ProofStorage;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<String>,
    pub sha256_hash: Vec<u64>,
    pub proof_index: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof {
    pub needs_change: bool,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub range_total_value: BigUint,
    pub validators_commitment: Vec<String>,
    pub balances_hash: Vec<u64>,
    pub withdrawal_credentials: Vec<Vec<u64>>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUint,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub proof_index: String,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FinalCircuitInput {
    #[serde(with = "bool_vec_as_int_vec")]
    pub state_root: Vec<bool>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub state_root_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub block_root: Vec<bool>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUint,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub slot_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub withdrawal_credentials: Vec<Vec<bool>>,
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
    pub block_root: Vec<u64>,
    pub withdrawal_credentials: Vec<Vec<u64>>,
    pub balance_sum: BigUint,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub proof: Vec<u8>,
}

pub trait NeedsChange {
    fn needs_change(&self) -> bool;
}

pub trait KeyProvider {
    fn get_key() -> String;
}

#[async_trait(?Send)]
pub trait ProofProvider {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8>;
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

#[async_trait(?Send)]
impl ProofProvider for ValidatorProof {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_index.clone())
            .await
            .unwrap()
    }
}

#[async_trait(?Send)]
impl ProofProvider for BalanceProof {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_index.clone())
            .await
            .unwrap()
    }
}

pub async fn fetch_validator_balance_input<const N: usize>(
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
    let json: String = con
        .get(VALIDATOR_COMMITMENT_CONSTANTS.final_proof_input_key)
        .await?;
    let result = serde_json::from_str::<FinalCircuitInput>(&json)?;
    Ok(result)
}

pub async fn save_balance_proof<const N: usize>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: u64,
    index: u64,
) -> Result<()> {
    let proof_index = format!(
        "{}:{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS.balance_verification_proof_storage, level, index
    );

    let balance_proof = BalanceProof {
        needs_change: false,
        range_total_value: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_range_total_value(&proof),
        balances_hash: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_range_balances_root(&proof).to_vec(),
        withdrawal_credentials: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::
            get_withdrawal_credentials(&proof)
            .map(|x| x.to_vec())
            .to_vec(),
        validators_commitment: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_range_validator_commitment(&proof).to_vec(),
        current_epoch: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_current_epoch(&proof),
        number_of_non_activated_validators: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_number_of_non_activated_validators(&proof),
        number_of_active_validators: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_number_of_active_validators(&proof),
        number_of_exited_validators: <ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2> as ValidatorBalanceProofExt<N>>::get_number_of_exited_validators(&proof),
        proof_index: proof_index.clone(),
    };

    proof_storage
        .set_proof(proof_index, &proof.to_bytes())
        .await?;

    save_json_object(
        con,
        &format!(
            "{}:{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .balance_verification_proof_key
                .to_owned(),
            level,
            index
        ),
        &balance_proof,
    )
    .await?;

    Ok(())
}

pub async fn save_final_proof(
    con: &mut Connection,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    block_root: Vec<u64>,
    withdrawal_credentials: Vec<Vec<u64>>,
    balance_sum: BigUint,
    number_of_non_activated_validators: u64,
    number_of_active_validators: u64,
    number_of_exited_validators: u64,
) -> Result<()> {
    let final_proof = FinalProof {
        needs_change: false,
        block_root,
        withdrawal_credentials: withdrawal_credentials,
        balance_sum: balance_sum,
        number_of_non_activated_validators: number_of_non_activated_validators,
        number_of_active_validators: number_of_active_validators,
        number_of_exited_validators: number_of_exited_validators,
        proof: proof.to_bytes(),
    };

    save_json_object(
        con,
        &VALIDATOR_COMMITMENT_CONSTANTS.final_layer_proof_key,
        &final_proof,
    )
    .await?;

    Ok(())
}

pub async fn delete_balance_verification_proof_dependencies(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    level: u64,
    index: u64,
) -> Result<()> {
    let proof_prefix = format!(
        "{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS
            .balance_verification_proof_storage
            .to_owned(),
        level - 1
    );

    let redis_prefix = format!(
        "{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS
            .balance_verification_proof_key
            .to_owned(),
        level - 1
    );

    con.del(format!("{}:{}", redis_prefix, index * 2)).await?;
    con.del(format!("{}:{}", redis_prefix, index * 2 + 1))
        .await?;

    let _ = proof_storage
        .del_proof(format!("{}:{}", proof_prefix, index * 2))
        .await;

    let _ = proof_storage
        .del_proof(format!("{}:{}", proof_prefix, index * 2 + 1))
        .await;

    if proof_storage
        .get_keys_count(format!(
            "{}:{}:*",
            VALIDATOR_COMMITMENT_CONSTANTS.balance_verification_proof_storage,
            level - 1
        ))
        .await
        == 1
    {
        con.del(format!("{}:{}", redis_prefix, VALIDATOR_REGISTRY_LIMIT))
            .await?;

        let _ = proof_storage
            .del_proof(format!("{}:{}", proof_prefix, VALIDATOR_REGISTRY_LIMIT))
            .await;
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
        validator_index,
    );

    let latest_epoch = get_latest_epoch(con, &key, epoch).await?;
    Ok(
        fetch_redis_json_object::<ValidatorShaInput>(con, format!("{}:{}", key, latest_epoch))
            .await?,
    )
}

pub async fn save_zero_validator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: u64,
) -> Result<()> {
    let proof_index = format!(
        "{}:zeroes:{}",
        VALIDATOR_COMMITMENT_CONSTANTS.validator_proof_storage, depth
    );

    let validator_proof = ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        needs_change: false,
        proof_index: proof_index.clone(),
    };

    proof_storage
        .set_proof(proof_index, &proof.to_bytes())
        .await?;

    save_json_object(
        con,
        &format!(
            "{}:zeroes:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_proof_key
                .to_owned(),
            depth,
        ),
        &validator_proof,
    )
    .await?;

    Ok(())
}

pub async fn save_json_object<T: Serialize>(
    con: &mut Connection,
    key: &str,
    object: &T,
) -> Result<()> {
    let json = serde_json::to_string(object)?;
    con.set(key, json).await?;
    Ok(())
}

pub fn u64_to_ssz_leaf(value: u64) -> [u8; 32] {
    let mut ret = vec![0u8; 32];
    ret[0..8].copy_from_slice(value.to_le_bytes().as_slice());
    ret.try_into().unwrap()
}

pub fn bits_to_bytes(bits: &[u64]) -> Vec<u8> {
    bits.chunks(8)
        .map(|bits| (0..8usize).fold(0u8, |byte, pos| byte | ((bits[pos]) << (7 - pos)) as u8))
        .collect::<Vec<_>>()
}

pub async fn save_validator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    gindex: u64,
    epoch: u64,
) -> Result<()> {
    let proof_index = format!(
        "{}:{}:{}",
        VALIDATOR_COMMITMENT_CONSTANTS.validator_proof_storage, gindex, epoch
    );
    let validator_proof = ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        proof_index: proof_index.clone(),
        needs_change: false,
    };

    proof_storage
        .set_proof(proof_index, &proof.to_bytes())
        .await?;

    // fetch validators len
    if gindex == 1 {
        let length: u64 = con
            .get_del(format!(
                "{}:{}",
                VALIDATOR_COMMITMENT_CONSTANTS.validators_length_key, epoch
            ))
            .await?;

        let validators_root_bytes: Vec<u8> = [
            &bits_to_bytes(&validator_proof.sha256_hash)[..],
            &u64_to_ssz_leaf(length)[..],
        ]
        .concat()
        .try_into()
        .unwrap();

        let validators_root = hex::encode(hash_bytes(validators_root_bytes.as_slice()));

        con.set(
            format!(
                "{}:{}",
                VALIDATOR_COMMITMENT_CONSTANTS.validators_root_key, epoch
            ),
            validators_root,
        )
        .await?;
    }

    save_json_object(
        con,
        &format!(
            "{}:{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_proof_key
                .to_owned(),
            gindex,
            epoch
        ),
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
    let json: String = con.get(key).await?;
    let result = serde_json::from_str::<T>(&json)?;
    Ok(result)
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
    proof_storage: &mut dyn ProofStorage,
    gindex: u64,
    epoch: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let left_child_gindex = gindex * 2;
    let right_child_gindex = gindex * 2 + 1;

    let proof1 = fetch_proof::<T>(con, left_child_gindex, epoch).await?;
    let proof2 = fetch_proof::<T>(con, right_child_gindex, epoch).await?;

    Ok((
        proof1.get_proof(proof_storage).await,
        proof2.get_proof(proof_storage).await,
    ))
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

pub async fn fetch_proof_balance<T: NeedsChange + KeyProvider + DeserializeOwned>(
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

// @TODO: Rename this later
pub async fn fetch_proofs_balances<
    T: NeedsChange + KeyProvider + ProofProvider + DeserializeOwned + Clone,
>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    level: u64,
    index: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (left_child_index, right_child_index) = (index * 2, index * 2 + 1);

    let proof1 = fetch_proof_balances::<T>(con, level - 1, left_child_index).await?;
    let proof2 = fetch_proof_balances::<T>(con, level - 1, right_child_index).await?;

    Ok((
        proof1.get_proof(proof_storage).await,
        proof2.get_proof(proof_storage).await,
    ))
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
