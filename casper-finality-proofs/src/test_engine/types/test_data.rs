use core::fmt::Debug;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestInput {
    pub a: u64,
    pub b: u64,
    pub outputs: Vec<u64>,
}
