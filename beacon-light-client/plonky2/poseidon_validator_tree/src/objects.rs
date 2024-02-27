use num_bigint::BigUint;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Validator {
    pub trusted: bool,
    pub validator_index: u64,

    pub pubkey: [bool; 384],
    pub withdrawal_credentials: [bool; 256],
    pub effective_balance: BigUint,
    pub slashed: bool,
    pub activation_eligibility_epoch: BigUint,
    pub activation_epoch: BigUint,
    pub exit_epoch: BigUint,
    pub withdrawable_epoch: BigUint,
}


#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorDataInput {
    pub trusted: bool,
    pub validator_index: u64,

    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub effective_balance: u64,
    pub exit_epoch: u64,
    pub pubkey: String,
    pub slashed: bool,
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidatorPoseidonDataOutput {
    pub trusted: bool,
    pub validator_index: u64,

    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub effective_balance: u64,
    pub exit_epoch: u64,
    pub pubkey: String,
    pub slashed: bool,
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
    pub validator_poseidon_hash: HashOut<GoldilocksField>,
    pub validator_poseidon_proof: Vec<HashOut<GoldilocksField>>
}
