use core::fmt::Debug;
use ethers::types::H256;
use primitive_types::H384;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Inputs {
    pub pubkey: H384,
    pub slashed: bool,
    pub a: H256,
    pub b: H256,
    pub slot: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Outputs {
    pub hash: H256,
    pub epoch: u64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestInput {
    pub inputs: Inputs,
    pub outputs: Outputs,
}
