use crate::{
    serializers::{biguint_to_str, parse_biguint}, validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel, withdrawal_credentials_balance_aggregator::first_level::WithdrawalCredentialsBalanceAggregatorFirstLevel
};
use circuit::CircuitOutput;
use num::{BigInt, BigUint};
use serde::{Deserialize, Serialize};

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
pub struct FinalProof {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BLSData {
    pub pubkey: String,
    pub signature: String,
    pub signing_root: String,
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub deposit_index: BigUint
}
