use core::fmt::Debug;
use ethers::types::H256;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Inputs {
    pub count: u64,
    pub seed: H256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Outputs {
    pub mapping: Vec<u64>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestData {
    pub inputs: Inputs,
    pub outputs: Outputs,
}
