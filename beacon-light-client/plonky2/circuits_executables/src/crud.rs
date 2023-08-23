use std::{fs, marker::PhantomData, thread, time::Duration};

use crate::{
    validator::{
        bool_vec_as_int_vec, bool_vec_as_int_vec_nested, ValidatorShaInput,
        VALIDATOR_REGISTRY_LIMIT,
    },
    validator_balances_input::{from_str, to_string, ValidatorBalancesInput},
    validator_commitment_constants::get_validator_commitment_constants,
};
use anyhow::Result;
use circuits::generator_serializer::{DendrETHGateSerializer, DendrETHGeneratorSerializer};
use num::BigUint;
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field64},
    plonk::{
        circuit_data::CircuitData, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs,
    },
};
use redis::{aio::Connection, AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};

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
    pub current_epoch: Vec<u64>,
    pub proof: Vec<u8>,
}

fn biguint_to_str<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str_value = value.to_str_radix(10);
    serializer.serialize_str(&str_value)
}

fn parse_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
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
    #[serde(serialize_with = "to_string", deserialize_with = "from_str")]
    pub withdrawal_credentials: Vec<u64>,
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
    pub balance_sum: u64,
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

pub async fn fetch_final_layer_input(con: &mut Connection) -> Result<FinalCircuitInput> {
    let json_str: String = con
        .get(get_validator_commitment_constants().final_proof_input_key)
        .await?;

    let final_layer_input: FinalCircuitInput = serde_json::from_str(&json_str)?;

    Ok(final_layer_input)
}

pub async fn save_balance_proof(
    con: &mut Connection,
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let balance_proof = serde_json::to_string(&BalanceProof {
        needs_change: false,
        range_total_value: proof.public_inputs[0].0 % GoldilocksField::ORDER,
        balances_hash: proof.public_inputs[1..257]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        withdrawal_credentials: proof.public_inputs[257..262]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        validators_commitment: proof.public_inputs[262..266]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        current_epoch: proof.public_inputs[266..268]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
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

pub async fn save_final_proof(
    con: &mut redis::aio::Connection,
    proof: &ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
) -> Result<()> {
    let final_proof = serde_json::to_string(&FinalProof {
        needs_change: false,
        state_root: proof.public_inputs[0..256]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        withdrawal_credentials: proof.public_inputs[256..261]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        balance_sum: proof.public_inputs[261].0 % GoldilocksField::ORDER,
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
    proof: ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>,
    depth: usize,
    index: usize,
) -> Result<()> {
    let validator_proof = serde_json::to_string(&ValidatorProof {
        poseidon_hash: proof.public_inputs[0..4]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
        sha256_hash: proof.public_inputs[4..260]
            .iter()
            .map(|x| x.0 % GoldilocksField::ORDER)
            .collect(),
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
