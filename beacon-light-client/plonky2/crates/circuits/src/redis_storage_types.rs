use crate::{
    validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
    withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel,
};
use circuit::{Array, CircuitOutput};
use plonky2::hash::hash_types::NUM_HASH_OUT_ELTS;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorsCommitmentMapperProofData {
    pub needs_change: bool,
    pub proof_key: String,
    pub public_inputs: CircuitOutput<ValidatorsCommitmentMapperFirstLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalCredentialsBalanceVerificationProofData<
    const VALIDATORS_COUNT: usize,
    const WITHDRAWAL_CREDENTIALS_COUNT: usize,
> where
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
pub struct BalanceVerificationFinalProofData {
    pub needs_change: bool,
    pub block_root: String,
    pub withdrawal_credentials: Vec<String>,
    pub balance_sum: u64,
    pub number_of_non_activated_validators: u64,
    pub number_of_active_validators: u64,
    pub number_of_exited_validators: u64,
    pub number_of_slashed_validators: u64,
    pub proof: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PubkeyCommitmentMapperRedisStorageData {
    pub sha256: String,
    pub poseidon: Array<u64, NUM_HASH_OUT_ELTS>,
    pub proof_key: String,
}
