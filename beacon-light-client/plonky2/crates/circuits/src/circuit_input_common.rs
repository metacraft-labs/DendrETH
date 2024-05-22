use crate::{
    final_layer::build_final_circuit::FinalCircuitTargets,
    serializers::{biguint_to_str, bool_vec_as_int_vec, bool_vec_as_int_vec_nested, parse_biguint},
    utils::biguint::WitnessBigUint,
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::CircuitOutput;
use num::BigUint;
use plonky2::{
    hash::hash_types::RichField,
    iop::{
        target::BoolTarget,
        witness::{PartialWitness, WitnessWrite},
    },
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorProof {
    pub needs_change: bool,
    pub proof_key: String,
    pub public_inputs: CircuitOutput<ValidatorsCommitmentMapperFirstLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceProof<const VALIDATORS_COUNT: usize, const WITHDRAWAL_CREDENTIALS_COUNT: usize>
where
    [(); VALIDATORS_COUNT / 4]:,
{
    pub needs_change: bool,
    pub proof_key: String,
    pub public_inputs: CircuitOutput<
        WithdrawalCredentialsBalanceAggregatorFirstLevel<
            VALIDATORS_COUNT,
            WITHDRAWAL_CREDENTIALS_COUNT,
        >,
    >,
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
    pub block_root: String,
    pub withdrawal_credentials: Vec<String>,
    pub balance_sum: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub proof: Vec<u8>,
}

pub fn set_boolean_pw_values<F: RichField>(
    pw: &mut PartialWitness<F>,
    target: &[BoolTarget],
    source: &Vec<bool>,
) {
    for i in 0..target.len() {
        pw.set_bool_target(target[i], source[i]);
    }
}

pub trait SetPWValues<F: RichField, T> {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &T);
}

impl<F: RichField, const N: usize> SetPWValues<F, FinalCircuitInput> for FinalCircuitTargets<N> {
    fn set_pw_values(&self, pw: &mut PartialWitness<F>, source: &FinalCircuitInput) {
        set_boolean_pw_values(pw, &self.state_root, &source.state_root);

        for i in 0..source.state_root_branch.len() {
            set_boolean_pw_values(pw, &self.state_root_branch[i], &source.state_root_branch[i]);
        }

        set_boolean_pw_values(pw, &self.block_root, &source.block_root);

        pw.set_biguint_target(&self.slot, &source.slot);

        for i in 0..source.slot_branch.len() {
            set_boolean_pw_values(pw, &self.slot_branch[i], &source.slot_branch[i]);
        }

        for i in 0..N {
            set_boolean_pw_values(
                pw,
                &self.withdrawal_credentials[i],
                &source.withdrawal_credentials[i],
            );
        }

        for i in 0..source.balance_branch.len() {
            set_boolean_pw_values(pw, &self.balance_branch[i], &source.balance_branch[i]);
        }

        for i in 0..source.validators_branch.len() {
            set_boolean_pw_values(pw, &self.validators_branch[i], &source.validators_branch[i]);
        }

        set_boolean_pw_values(pw, &self.validator_size_bits, &source.validators_size_bits);
    }
}

pub fn from_str<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Vec<String> = Vec::deserialize(deserializer)?;
    s.into_iter()
        .map(|str_val| str_val.parse::<u64>().map_err(serde::de::Error::custom))
        .collect()
}

pub fn to_string<S>(x: &Vec<u64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let string_vec: Vec<String> = x.iter().map(|&num| num.to_string()).collect();
    string_vec.serialize(s)
}
