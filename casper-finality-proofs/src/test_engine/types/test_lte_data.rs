use core::fmt::Debug;
use ethers::types::U256;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    pub a: U256,
    pub b: U256,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestInput {
    pub inputs: Inputs,
}
