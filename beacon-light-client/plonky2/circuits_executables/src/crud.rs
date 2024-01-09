use std::{fs, marker::PhantomData};

use crate::{
    validator::{
        bool_vec_as_int_vec, bool_vec_as_int_vec_nested, ValidatorShaInput,
        VALIDATOR_REGISTRY_LIMIT,
    },
    validator_balances_input::ValidatorBalancesInput,
    validator_commitment_constants::VALIDATOR_COMMITMENT_CONSTANTS,
};
use anyhow::Result;
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

#[derive(Debug, Deserialize)]
pub struct ProofTaskData {
    pub level: usize,
    pub left_proof_index: usize,
    pub right_proof_index: usize,
}

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

#[derive(Serialize, Deserialize, Debug)]
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
    index: usize,
) -> Result<ValidatorBalancesInput> {
    let json_str: String = con
        .get(format!(
            "{}:{}",
            VALIDATOR_COMMITMENT_CONSTANTS
                .validator_balance_input_key
                .to_owned(),
            index
        ))
        .await?;

    let validator_balance_input: ValidatorBalancesInput = serde_json::from_str(&json_str)?;

    Ok(validator_balance_input)
}

pub async fn fetch_final_layer_input(con: &mut Connection) -> Result<FinalCircuitInput> {
    let json_str: String = con
        .get(
            VALIDATOR_COMMITMENT_CONSTANTS
                .final_proof_input_key
                .to_owned(),
        )
        .await?;

    let final_layer_input: FinalCircuitInput = serde_json::from_str(&json_str)?;

    Ok(final_layer_input)
}

pub async fn save_balance_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: usize,
    index: usize,
) -> Result<()> {
    let balance_proof = serde_json::to_string(&BalanceProof {
        needs_change: false,
        range_total_value: proof.get_range_total_value(),
        balances_hash: proof.get_range_balances_root().to_vec(),
        withdrawal_credentials: proof.get_withdrawal_credentials(),
        validators_commitment: proof.get_range_validator_commitment().to_vec(),
        current_epoch: proof.get_current_epoch(),
        proof: proof.to_bytes(),
    })?;

    let _: () = con
        .set(
            format!(
                "{}:{}:{}",
                VALIDATOR_COMMITMENT_CONSTANTS
                    .balance_verification_proof_key
                    .to_owned(),
                level,
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
        withdrawal_credentials: proof.get_final_circuit_withdrawal_credentials(),
        balance_sum: proof.get_final_circuit_balance_sum(),
        proof: proof.to_bytes(),
    })?;

    let _: () = con
        .set(
            VALIDATOR_COMMITMENT_CONSTANTS
                .final_layer_proof_key
                .to_owned(),
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
            VALIDATOR_COMMITMENT_CONSTANTS.validator_key.to_owned(),
            validator_index
        ))
        .await?;
    let validator: ValidatorShaInput = serde_json::from_str(&json_str)?;

    Ok(validator)
}

pub async fn save_validator_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    level: usize,
    index: usize,
) -> Result<()> {
    let validator_proof = serde_json::to_string(&ValidatorProof {
        poseidon_hash: proof
            .get_commitment_mapper_poseidon_hash_tree_root()
            .to_vec(),
        sha256_hash: proof.get_commitment_mapper_sha256_hash_tree_root().to_vec(),
        proof: proof.to_bytes(),
        needs_change: false,
    })?;

    let _: () = con
        .set(
            format!(
                "{}:{}:{}",
                VALIDATOR_COMMITMENT_CONSTANTS
                    .validator_proof_key
                    .to_owned(),
                level,
                index
            ),
            validator_proof,
        )
        .await?;

    Ok(())
}

pub async fn fetch_proof<T: NeedsChange + KeyProvider + DeserializeOwned>(
    con: &mut Connection,
    level: usize,
    index: usize,
) -> Result<T> {
    let mut retries = 0;

    loop {
        if retries > 5 {
            return Err(anyhow::anyhow!("Not able to complete, try again"));
        }

        let mut proof_result: Result<String, RedisError> = con
            .get(format!("{}:{}:{}", T::get_key(), level, index))
            .await;

        if proof_result.is_err() {
            // get the zeroth proof
            proof_result = con
                .get(format!(
                    "{}:{}:{}",
                    T::get_key(),
                    level,
                    VALIDATOR_REGISTRY_LIMIT
                ))
                .await;
        }

        let proof = serde_json::from_str::<T>(&proof_result?)?;

        if proof.needs_change() {
            // Wait a bit and try again
            // thread::sleep(Duration::from_secs(10));
            retries += 1;

            continue;
        }

        return Ok(proof);
    }
}

pub async fn fetch_proofs<T: NeedsChange + KeyProvider + ProofProvider + DeserializeOwned>(
    con: &mut Connection,
    task_data: &ProofTaskData,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let proof1 = fetch_proof::<T>(con, task_data.level, task_data.left_proof_index).await?;
    let proof2 = fetch_proof::<T>(con, task_data.level, task_data.right_proof_index).await?;

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
