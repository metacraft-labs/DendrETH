use core::fmt::Debug;
use primitive_types::H256;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProveFinalityData {
    pub slot: u64,
    pub total_number_of_validators: u64,
    pub previous_epoch_attested_validators: u64,
    pub current_epoch_attested_validators: u64,
    pub source: CheckpointBlockData,
    pub target: CheckpointBlockData,
    pub justification_bits: Vec<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CheckpointBlockData {
    pub epoch: u64,
    pub root: H256,
}
