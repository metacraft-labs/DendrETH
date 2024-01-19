use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ValidatorData {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: u8,
    pub activation_epoch: u8,
    pub exit_epoch: u64,
    pub withdrawable_epoch: u64,
}

// #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
// pub struct ValidatorData1 {
//     pub pubkey: String,
//     pub withdrawal_credentials: String,
//     pub effective_balance: u64,
//     pub slashed: bool,
//     pub activation_eligibility_epoch: u64,
//     pub activation_epoch: u64,
//     pub exit_epoch: u64,
//     pub withdrawable_epoch: u64,
// }

#[derive(Debug, Deserialize)]
pub struct ValidatorIndexData {
    #[serde(flatten)]
    data: HashMap<String, ValidatorData>,
}
