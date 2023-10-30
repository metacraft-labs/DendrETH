use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    validator::{
        bool_vec_as_int_vec, bool_vec_as_int_vec_nested, ValidatorShaInput,
        VALIDATOR_REGISTRY_LIMIT,
    },
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::get_validator_commitment_constants,
};
use anyhow::{Ok, Result};
use async_trait::async_trait;

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
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

use super::proof_storage::proof_storage::ProofStorage;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub poseidon_hash: Vec<u64>,
    pub sha256_hash: Vec<u64>,
    pub proof_index: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof {
    pub needs_change: bool,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub range_total_value: BigUint,
    pub validators_commitment: Vec<u64>,
    pub balances_hash: Vec<u64>,
    pub withdrawal_credentials: Vec<u64>,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUint,
    pub proof_index: String,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
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
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub slot: BigUint,
    #[serde(with = "bool_vec_as_int_vec_nested")]
    pub slot_branch: Vec<Vec<bool>>,
    #[serde(with = "bool_vec_as_int_vec")]
    pub withdrawal_credentials: Vec<bool>,
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
    pub withdrawal_credentials: Vec<u64>,
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

pub async fn fetch_final_layer_input(con: &mut Connection) -> Result<FinalCircuitInput> {
    let json_str: String = con
        .get(get_validator_commitment_constants().final_proof_input_key)
        .await?;

    let final_layer_input: FinalCircuitInput = serde_json::from_str(&json_str)?;

    Ok(final_layer_input)
}

pub async fn save_balance_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let proof_index = format!(
        "{}:{}:{}",
        "balance_verification_proof_storage", depth, index
    );

    let balance_proof = serde_json::to_string(&BalanceProof {
        needs_change: false,
        range_total_value: proof.get_range_total_value(),
        balances_hash: proof.get_range_balances_root().to_vec(),
        withdrawal_credentials: proof.get_withdrawal_credentials().to_vec(),
        validators_commitment: proof.get_range_validator_commitment().to_vec(),
        current_epoch: proof.get_current_epoch(),
        proof_index: proof_index.clone(),
        number_of_non_activated_validators: proof.get_number_of_non_activated_validators(),
        number_of_active_validators: proof.get_number_of_active_validators(),
        number_of_exited_validators: proof.get_number_of_exited_validators(),
    })?;

    proof_storage
        .set_proof(proof_index, &proof.to_bytes())
        .await?;

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

pub async fn save_final_proof(
    con: &mut redis::aio::Connection,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<()> {
    let final_proof = serde_json::to_string(&FinalProof {
        needs_change: false,
        state_root: proof.get_final_circuit_state_root().to_vec(),
        withdrawal_credentials: proof.get_final_circuit_withdrawal_credentials().to_vec(),
        balance_sum: proof.get_final_circuit_balance_sum(),
        number_of_non_activated_validators: proof.get_final_number_of_non_activated_validators(),
        number_of_active_validators: proof.get_final_number_of_active_validators(),
        number_of_exited_validators: proof.get_final_number_of_exited_validators(),
        proof: proof.to_bytes(),
    })?;

    let _: () = con
        .set(
            get_validator_commitment_constants().final_layer_proof_key,
            final_proof,
        )
        .await?;

    Ok(())
}

pub async fn fetch_validator(
    con: &mut Connection,
    validator_index: usize,
) -> Result<ValidatorShaInput> {
    let json_str: String = con
        .get(format!(
            "{}:{}",
            get_validator_commitment_constants().validator_key,
            validator_index
        ))
        .await?;
    let validator: ValidatorShaInput = serde_json::from_str(&json_str)?;

    Ok(validator)
}

pub async fn save_validator_proof(
    con: &mut Connection,
    proof_storage: &mut dyn ProofStorage,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let proof_index = format!("{}:{}:{}", "validator_proof_storage", depth, index);

    let validator_proof = serde_json::to_string(&ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        proof_index: proof_index.clone(),
        needs_change: false,
    })?;

    proof_storage
        .set_proof(proof_index, &proof.to_bytes())
        .await?;

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
    proof_storage: &mut dyn ProofStorage,
    indexes: &Vec<usize>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let proof1 = fetch_proof::<T>(con, indexes[0], indexes[1]).await?;
    let proof2 = fetch_proof::<T>(con, indexes[0], indexes[2]).await?;

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
