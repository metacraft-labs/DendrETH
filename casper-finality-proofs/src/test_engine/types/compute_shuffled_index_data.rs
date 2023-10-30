use core::fmt::Debug;
use ethers::types::H256;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestData {
    pub count: u64,
    pub seed: H256,
    pub mapping: Vec<u64>,
}
