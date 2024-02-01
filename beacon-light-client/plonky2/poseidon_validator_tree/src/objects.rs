use num_bigint::BigUint;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Validator {
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
    pub activation_eligibility_epoch: u64,
    pub activation_epoch: u64,
    pub effective_balance: u64,
    pub exit_epoch: u64,
    pub pubkey: String,
    pub slashed: bool,
    pub withdrawable_epoch: u64,
    pub withdrawal_credentials: String,
}
