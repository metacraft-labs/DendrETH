use std::fmt::Display;

use num::FromPrimitive;
use num_derive::FromPrimitive;

use crate::validator::VALIDATOR_REGISTRY_LIMIT;

#[derive(FromPrimitive)]
#[repr(u8)]
enum CommitmentMapperTaskType {
    UpdateProofNode,
    ProveZeroForDepth,
    UpdateValidatorProof,
}

type Gindex = u64;
type Epoch = u64;
type ValidatorIndex = u64;
type Depth = u64;

#[derive(Debug)]
pub enum CommitmentMapperTask {
    UpdateProofNode(Gindex, Epoch),
    ProveZeroForDepth(Depth),
    UpdateValidatorProof(ValidatorIndex, Epoch),
}

impl CommitmentMapperTask {
    pub fn log(&self) {
        match *self {
            CommitmentMapperTask::UpdateProofNode(gindex, epoch) => println!(
                "Updating proof node at gindex {} for epoch {}...",
                gindex, epoch
            ),
            CommitmentMapperTask::ProveZeroForDepth(depth) => {
                println!("Proving zero for depth {}...", depth)
            }
            CommitmentMapperTask::UpdateValidatorProof(validator_index, epoch) => {
                if validator_index != VALIDATOR_REGISTRY_LIMIT as u64 {
                    println!(
                        "Updating validator proof at index {} for epoch {}...",
                        validator_index, epoch
                    );
                } else {
                    println!("Proving zero validator...");
                }
            }
        };
    }
}

impl Display for CommitmentMapperTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            CommitmentMapperTask::UpdateProofNode(gindex, epoch) => f.write_str(&format!(
                "UpdateProofNode(gindex = {}, epoch = {})",
                gindex, epoch
            )),
            CommitmentMapperTask::ProveZeroForDepth(_) => f.write_str(&format!("{:?}", *self)),
            CommitmentMapperTask::UpdateValidatorProof(validator_index, epoch) => {
                f.write_str(&format!(
                    "UpdateValidatorProof(validator_index = {}, epoch = {})",
                    validator_index, epoch
                ))
            }
        }
    }
}

pub fn deserialize_task(bytes: &[u8]) -> Option<CommitmentMapperTask> {
    match FromPrimitive::from_u8(u8::from_be_bytes(bytes[0..1].try_into().unwrap()))? {
        CommitmentMapperTaskType::UpdateProofNode => {
            let gindex = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            let epoch = u64::from_be_bytes(bytes[9..17].try_into().unwrap());
            Some(CommitmentMapperTask::UpdateProofNode(gindex, epoch))
        }
        CommitmentMapperTaskType::ProveZeroForDepth => {
            let depth = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            Some(CommitmentMapperTask::ProveZeroForDepth(depth))
        }
        CommitmentMapperTaskType::UpdateValidatorProof => {
            let validator_index = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
            let epoch = u64::from_be_bytes(bytes[9..17].try_into().unwrap());
            Some(CommitmentMapperTask::UpdateValidatorProof(
                validator_index,
                epoch,
            ))
        }
    }
}
