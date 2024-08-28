use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    constants::VALIDATOR_REGISTRY_LIMIT, db_constants::DB_CONSTANTS, utils::get_depth_for_gindex,
};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use circuit::{Circuit, CircuitInput, SerdeCircuitTarget};
use circuits::{
    bls_verification::build_stark_proof_verifier::RecursiveStarkTargets,
    deposit_accumulator_balance_aggregator_diva::{
        final_layer::DepositAccumulatorBalanceAggregatorDivaFinalLayer,
        first_level::DepositAccumulatorBalanceAggregatorDivaFirstLevel,
    },
    redis_storage_types::{
        BalanceVerificationFinalProofData, DepositAccumulatorBalanceAggregatorDivaProofData,
        DepositAccumulatorFinalProofData, PubkeyCommitmentMapperRedisStorageData,
        ValidatorsCommitmentMapperProofData, WithdrawalCredentialsBalanceVerificationProofData,
    },
    utils::{bits_to_bytes, hash_bytes, u64_to_ssz_leaf},
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::{
        final_layer::BalanceVerificationFinalCircuit,
        first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
    },
};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    plonk::{
        circuit_data::{CircuitData, CommonCircuitData},
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputs,
    },
    util::serialization::Buffer,
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
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

impl NeedsChange for ValidatorsCommitmentMapperProofData {
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl KeyProvider for ValidatorsCommitmentMapperProofData {
    fn get_key() -> String {
        DB_CONSTANTS.validator_proof_key.to_owned()
    }
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> NeedsChange
    for WithdrawalCredentialsBalanceVerificationProofData<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> KeyProvider
    for WithdrawalCredentialsBalanceVerificationProofData<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
where
    [(); VALIDATORS_COUNT / 4]:,
{
    fn get_key() -> String {
        DB_CONSTANTS.balance_verification_proof_key.to_owned()
    }
}

impl NeedsChange for DepositAccumulatorBalanceAggregatorDivaProofData {
    fn needs_change(&self) -> bool {
        self.needs_change
    }
}

impl KeyProvider for DepositAccumulatorBalanceAggregatorDivaProofData {
    fn get_key() -> String {
        DB_CONSTANTS
            .deposit_balance_verification_proof_key
            .to_owned()
    }
}

#[async_trait(?Send)]
impl ProofProvider for ValidatorsCommitmentMapperProofData {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_key.clone())
            .await
            .unwrap()
    }
}

#[async_trait(?Send)]
impl<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize> ProofProvider
    for WithdrawalCredentialsBalanceVerificationProofData<
        VALIDATORS_COUNT,
        WITHDRAWAL_CREDENTIALS_COUNT,
    >
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

#[async_trait(?Send)]
impl ProofProvider for DepositAccumulatorBalanceAggregatorDivaProofData {
    async fn get_proof(&self, proof_storage: &mut dyn ProofStorage) -> Vec<u8> {
        proof_storage
            .get_proof(self.proof_key.clone())
            .await
            .unwrap()
    }
}

pub async fn fetch_validator_balance_aggregator_input(
    con: &mut Connection,
    protocol: String,
    index: u64,
) -> Result<CircuitInput<DepositAccumulatorBalanceAggregatorDivaFirstLevel>> {
    Ok(fetch_redis_json_object(
        con,
        format!(
            "{}:{}:{}",
            protocol,
            DB_CONSTANTS
                .deposit_balance_verification_input_key
                .to_owned(),
            index
        ),
    )
    .await?)
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

pub async fn fetch_final_layer_input<const WITHDRAWAL_CREDENTIALS_COUNT: usize>(
    con: &mut Connection,
    protocol: &str,
) -> Result<CircuitInput<BalanceVerificationFinalCircuit<WITHDRAWAL_CREDENTIALS_COUNT>>> {
    let json: String = con
        .get(format!(
            "{}:{}",
            protocol, DB_CONSTANTS.final_proof_input_key
        ))
        .await?;
    Ok(serde_json::from_str(&json)?)
}

pub async fn fetch_deposit_accumulator_final_layer_input(
    con: &mut Connection,
    protocol: &str,
) -> Result<CircuitInput<DepositAccumulatorBalanceAggregatorDivaFinalLayer>> {
    let json: String = con
        .get(format!(
            "{}:{}",
            protocol, DB_CONSTANTS.deposit_balance_verification_final_proof_input_key
        ))
        .await?;

    Ok(serde_json::from_str(&json)?)
}

pub async fn save_balance_aggregator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    protocol: String,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: u64,
    index: u64,
) -> Result<()> {
    let proof_key = format!(
        "{}:{}:{}:{}",
        protocol, DB_CONSTANTS.deposit_balance_verification_proof_key, level, index
    );

    let public_inputs =
        DepositAccumulatorBalanceAggregatorDivaFirstLevel::read_public_inputs(&proof.public_inputs);

    let balance_aggregator_proof = DepositAccumulatorBalanceAggregatorDivaProofData {
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
            DB_CONSTANTS
                .deposit_balance_verification_proof_key
                .to_owned(),
            level,
            index
        ),
        &balance_aggregator_proof,
    )
    .await?;

    Ok(())
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

    let balance_proof = WithdrawalCredentialsBalanceVerificationProofData {
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
    number_of_slashed_validators: u64,
) -> Result<()> {
    let final_proof = BalanceVerificationFinalProofData {
        needs_change: false,
        block_root,
        withdrawal_credentials,
        balance_sum,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
        number_of_slashed_validators,
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

pub async fn save_deposit_accumulator_final_proof(
    con: &mut Connection,
    protocol: String,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    slot: u64,
    block_number: u64,
    block_root: String,
    balance_sum: u64,
    number_of_non_activated_validators: u64,
    number_of_active_validators: u64,
    number_of_exited_validators: u64,
    number_of_slashed_validators: u64,
) -> Result<()> {
    let final_proof = DepositAccumulatorFinalProofData {
        needs_change: false,
        slot,
        block_number,
        block_root,
        balance_sum,
        number_of_non_activated_validators,
        number_of_active_validators,
        number_of_exited_validators,
        number_of_slashed_validators,
        proof: proof.to_bytes(),
    };

    save_json_object(
        con,
        &format!(
            "{}:{}",
            protocol, &DB_CONSTANTS.deposit_balance_verification_final_proof_key
        ),
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

pub async fn delete_balance_verification_diva_proof_dependencies(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    protocol: &str,
    level: u64,
    index: u64,
) -> Result<()> {
    let proof_prefix = format!(
        "{}:{}:{}",
        protocol,
        DB_CONSTANTS
            .deposit_balance_verification_proof_key
            .to_owned(),
        level - 1,
    );

    let redis_prefix = format!(
        "{}:{}:{}",
        protocol,
        DB_CONSTANTS
            .deposit_balance_verification_proof_key
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

pub fn get_block_number_with_latest_change(keys: Vec<u64>, block_number: u64) -> Result<u64> {
    let mut min_key = keys[0];
    let mut min_key_difference = u64::MAX;

    for i in 0..keys.len() {
        let key = keys[i];

        if block_number >= key && block_number - key < min_key_difference {
            min_key = key;
            min_key_difference = block_number - key;
        }
    }

    if min_key_difference == u64::MAX {
        return Err(anyhow::anyhow!("Could not find data for block number"));
    }

    Ok(min_key)
}

pub async fn fetch_pubkey_commitment_mapper_proof(
    con: &mut Connection,
    protocol: &str,
    block_number: u64,
) -> Result<PubkeyCommitmentMapperRedisStorageData> {
    let keys: Vec<String> = con
        .keys(format!("{protocol}:pubkey_commitment_mapper:root_proofs:*").as_str())
        .await?;

    let keys = keys
        .iter()
        .map(|key| key.split(":").last().unwrap().parse().unwrap())
        .collect();

    let key = get_block_number_with_latest_change(keys, block_number)?;

    Ok(fetch_redis_json_object(
        con,
        format!("{}:pubkey_commitment_mapper:root_proofs:{}", protocol, key),
    )
    .await?)
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

    let validator_proof = ValidatorsCommitmentMapperProofData {
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

async fn save_vcm_proof_data(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    proof_key: &str,
    gindex: u64,
    slot: u64,
) -> Result<()> {
    let proof_data = ValidatorsCommitmentMapperProofData {
        proof_key: proof_key.to_owned(),
        needs_change: false,
        public_inputs: ValidatorsCommitmentMapperFirstLevel::read_public_inputs(
            &proof.public_inputs,
        ),
    };

    // fetch validators len
    if gindex == 1 {
        let length: u64 = con
            .get(format!("{}:{}", DB_CONSTANTS.validators_length_key, slot))
            .await?;

        let validators_root_bytes: Vec<u8> = [
            &bits_to_bytes(&proof_data.public_inputs.sha256_hash_tree_root[..])[..],
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
        &proof_data,
    )
    .await?;

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

    proof_storage
        .set_proof(proof_key.clone(), &proof.to_bytes())
        .await?;

    save_vcm_proof_data(con, proof, &proof_key, gindex, slot).await?;

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

pub fn load_circuit_data<T: Circuit>(dir: &str, name: &str) -> Result<CircuitData<T::F, T::C, 2>>
where
    <<T as Circuit>::C as GenericConfig<2>>::Hasher: AlgebraicHasher<<T as Circuit>::F>,
    <T as Circuit>::C: 'static,
{
    let gate_serializer = CustomGateSerializer;
    let generator_serializer = CustomGeneratorSerializer {
        _phantom: PhantomData::<T::C>,
    };

    let circuit_data_bytes = read_from_file(&format!("{dir}/{name}.plonky2_circuit"))?;

    Ok(CircuitData::<T::F, T::C, 2>::from_bytes(
        &circuit_data_bytes,
        &gate_serializer,
        &generator_serializer,
    )
    .unwrap())
}

pub fn load_circuit_data_starky(
    file_name: &str,
) -> CircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2> {
    let circuit_data_bytes = read_from_file(&format!("{file_name}.plonky2_circuit")).unwrap();

    CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        &circuit_data_bytes,
        &CustomGateSerializer,
        &CustomGeneratorSerializer {
            _phantom: PhantomData::<PoseidonGoldilocksConfig>,
        },
    )
    .unwrap()
}

pub fn load_common_circuit_data_starky(file_name: &str) -> CommonCircuitData<GoldilocksField, 2> {
    let circuit_data_bytes = read_from_file(&format!("{file_name}.plonky2_common_data")).unwrap();

    CommonCircuitData::<GoldilocksField, 2>::from_bytes(circuit_data_bytes, &CustomGateSerializer)
        .unwrap()
}

pub fn get_recursive_stark_targets(
    file_name: &str,
) -> Result<RecursiveStarkTargets, anyhow::Error> {
    let target_bytes = read_from_file(&format!("{file_name}.plonky2_targets"))?;

    let mut target_buffer = Buffer::new(&target_bytes);

    Ok(RecursiveStarkTargets::deserialize(&mut target_buffer).unwrap())
}
