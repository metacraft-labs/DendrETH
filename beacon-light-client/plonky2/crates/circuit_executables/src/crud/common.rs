use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    db_constants::DB_CONSTANTS, utils::get_depth_for_gindex, validator::VALIDATOR_REGISTRY_LIMIT,
};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use circuit::{Circuit, CircuitInput};
use circuits::{
    circuit_input_common::{BalanceProof, FinalCircuitInput, FinalProof, ValidatorProof},
    serialization::generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer},
    utils::utils::{bits_to_bytes, hash_bytes, u64_to_ssz_leaf},
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};

use super::proof_storage::proof_storage::ProofStorage;

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
        DB_CONSTANTS.validator_proof_key.to_owned()
    }
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> NeedsChange
    for BalanceProof<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> KeyProvider
    for BalanceProof<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn get_key() -> String {
        DB_CONSTANTS.balance_verification_proof_key.to_owned()
    }
}

#[async_trait(?Send)]
impl ProofProvider for ValidatorProof {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_key.clone())
            .await
            .unwrap()
    }
}

#[async_trait(?Send)]
impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> ProofProvider
    for BalanceProof<VALIDATORS_COUNT, WITHDRAWAL_CREDENTIALS_COUNT>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_key.clone())
            .await
            .unwrap()
    }
}

pub async fn fetch_validator_balance_input<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    con: &mut Connection,
    protocol: String,
    index: u64,
) -> Result<
    CircuitInput<
        WithdrawalCredentialsBalanceAggregatorFirstLevel<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >,
    >,
>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    Ok(fetch_redis_json_object(
        con,
        format!(
            "{}:{}:{}",
            protocol,
            DB_CONSTANTS.validator_balance_input_key.to_owned(),
            index
        ),
    )
    .await?)
}

pub async fn fetch_final_layer_input(
    con: &mut Connection,
    protocol: &str,
) -> Result<FinalCircuitInput> {
    let json: String = con
        .get(format!(
            "{}:{}",
            protocol, DB_CONSTANTS.final_proof_input_key
        ))
        .await?;
    let result = serde_json::from_str::<FinalCircuitInput>(&json)?;
    Ok(result)
}

pub async fn save_balance_proof<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
>(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    protocol: String,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: u64,
    index: u64,
) -> Result<()>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    let proof_key = format!(
        "{}:{}:{}:{}",
        protocol, DB_CONSTANTS.balance_verification_proof_storage, level, index
    );

    let public_inputs = WithdrawalCredentialsBalanceAggregatorFirstLevel::<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >::read_public_inputs(&proof.public_inputs);

    let balance_proof = BalanceProof {
        needs_change: false,
        proof_key: proof_key.clone(),
        public_inputs,
    };

    proof_storage
        .set_proof(proof_key, &proof.to_bytes())
        .await?;

    save_json_object(
        con,
        &format!(
            "{}:{}:{}:{}",
            protocol,
            DB_CONSTANTS.balance_verification_proof_key.to_owned(),
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
    protocol: String,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    block_root: String,
    withdrawal_credentials: Vec<String>,
    balance_sum: u64,
    number_of_non_activated_validators: u64,
    number_of_active_validators: u64,
    number_of_exited_validators: u64,
) -> Result<()> {
    let final_proof = FinalProof {
        needs_change: false,
        block_root,
        withdrawal_credentials,
        balance_sum,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
        proof: proof.to_bytes(),
    };

    save_json_object(
        con,
        format!("{}:{}", protocol, &DB_CONSTANTS.final_layer_proof_key).as_str(),
        &final_proof,
    )
    .await?;

    Ok(())
}

pub async fn delete_balance_verification_proof_dependencies(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    protocol: &str,
    level: u64,
    index: u64,
) -> Result<()> {
    let proof_prefix = format!(
        "{}:{}:{}",
        protocol,
        DB_CONSTANTS.balance_verification_proof_storage.to_owned(),
        level - 1,
    );

    let redis_prefix = format!(
        "{}:{}:{}",
        protocol,
        DB_CONSTANTS.balance_verification_proof_key.to_owned(),
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

    if proof_storage.get_keys_count(proof_prefix.to_string()).await == 1 {
        con.del(format!("{}:{}", redis_prefix, VALIDATOR_REGISTRY_LIMIT))
            .await?;

        let _ = proof_storage
            .del_proof(format!("{}:{}", proof_prefix, VALIDATOR_REGISTRY_LIMIT))
            .await;
    }

    Ok(())
}

pub async fn get_slot_with_latest_change(
    con: &mut Connection,
    key: &String,
    slot: u64,
) -> Result<String> {
    let result: Vec<String> = con
        .zrevrangebyscore_limit(
            format!("{}:{}", key, DB_CONSTANTS.slot_lookup_key.to_owned(),),
            slot,
            0,
            0,
            1,
        )
        .await?;

    ensure!(!result.is_empty(), "Could not find data for slot");
    Ok(result[0].clone())
}

pub async fn fetch_validator(
    con: &mut Connection,
    validator_index: u64,
    slot: u64,
) -> Result<CircuitInput<ValidatorsCommitmentMapperFirstLevel>> {
    let key = format!(
        "{}:{}",
        DB_CONSTANTS.validator_key.to_owned(),
        validator_index,
    );

    let latest_change_slot = get_slot_with_latest_change(con, &key, slot).await?;
    Ok(fetch_redis_json_object(con, format!("{}:{}", key, latest_change_slot)).await?)
}

pub async fn save_zero_validator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: u64,
) -> Result<()> {
    let proof_key = format!("{}:zeroes:{}", DB_CONSTANTS.validator_proof_storage, depth);

    let validator_proof = ValidatorProof {
        needs_change: false,
        proof_key: proof_key.clone(),
        public_inputs: ValidatorsCommitmentMapperFirstLevel::read_public_inputs(
            &proof.public_inputs,
        ),
    };

    proof_storage
        .set_proof(proof_key, &proof.to_bytes())
        .await?;

    save_json_object(
        con,
        &format!(
            "{}:zeroes:{}",
            DB_CONSTANTS.validator_proof_key.to_owned(),
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

pub async fn save_validator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    gindex: u64,
    slot: u64,
) -> Result<()> {
    let proof_key = format!(
        "{}:{}:{}",
        DB_CONSTANTS.validator_proof_storage, gindex, slot
    );
    let validator_proof = ValidatorProof {
        proof_key: proof_key.clone(),
        needs_change: false,
        public_inputs: ValidatorsCommitmentMapperFirstLevel::read_public_inputs(
            &proof.public_inputs,
        ),
    };

    proof_storage
        .set_proof(proof_key, &proof.to_bytes())
        .await?;

    // fetch validators len
    if gindex == 1 {
        let length: u64 = con
            .get(format!("{}:{}", DB_CONSTANTS.validators_length_key, slot))
            .await?;

        let validators_root_bytes: Vec<u8> = [
            &bits_to_bytes(&validator_proof.public_inputs.sha256_hash_tree_root[..])[..],
            &u64_to_ssz_leaf(length)[..],
        ]
        .concat()
        .try_into()
        .unwrap();

        let validators_root = hex::encode(hash_bytes(validators_root_bytes.as_slice()));

        con.set(
            format!("{}:{}", DB_CONSTANTS.validators_root_key, slot),
            validators_root,
        )
        .await?;
    }

    save_json_object(
        con,
        &format!(
            "{}:{}:{}",
            DB_CONSTANTS.validator_proof_key.to_owned(),
            gindex,
            slot
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
    let json: String = con.get(&key).await?;
    let result = serde_json::from_str::<T>(&json)?;
    Ok(result)
}

pub async fn fetch_proof<T: NeedsChange + KeyProvider + DeserializeOwned + Clone>(
    con: &mut Connection,
    gindex: u64,
    slot: u64,
) -> Result<T> {
    let key = format!("{}:{}", T::get_key(), gindex);
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let latest_change_slot_result = get_slot_with_latest_change(con, &key, slot).await;

        let proof = match latest_change_slot_result {
            Ok(latest_change_slot) => {
                let proof_result =
                    fetch_redis_json_object::<T>(con, format!("{}:{}", key, latest_change_slot))
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
    slot: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let left_child_gindex = gindex * 2;
    let right_child_gindex = gindex * 2 + 1;

    let proof1 = fetch_proof::<T>(con, left_child_gindex, slot).await?;
    let proof2 = fetch_proof::<T>(con, right_child_gindex, slot).await?;

    Ok((
        proof1.get_proof(proof_storage).await,
        proof2.get_proof(proof_storage).await,
    ))
}

// @TODO: Rename this later
pub async fn fetch_proof_balances<T: NeedsChange + KeyProvider + DeserializeOwned + Clone>(
    con: &mut Connection,
    protocol: &str,
    level: u64,
    index: u64,
) -> Result<T> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result = fetch_redis_json_object::<T>(
            con,
            format!("{}:{}:{}:{}", protocol, T::get_key(), level, index),
        )
        .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = fetch_redis_json_object::<T>(
                con,
                format!(
                    "{}:{}:{}:{}",
                    protocol,
                    T::get_key(),
                    level,
                    VALIDATOR_REGISTRY_LIMIT
                ),
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
    protocol: String,
    depth: usize,
    index: usize,
) -> Result<T> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result: Result<String, RedisError> = con
            .get(format!("{}:{}:{}:{}", protocol, T::get_key(), depth, index))
            .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = con
                .get(format!(
                    "{}:{}:{}:{}",
                    protocol,
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
    protocol: String,
    level: u64,
    index: u64,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let (left_child_index, right_child_index) = (index * 2, index * 2 + 1);

    let proof1 = fetch_proof_balances::<T>(con, &protocol, level - 1, left_child_index).await?;
    let proof2 = fetch_proof_balances::<T>(con, &protocol, level - 1, right_child_index).await?;

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
