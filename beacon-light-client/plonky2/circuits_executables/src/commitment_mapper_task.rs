use colored::Colorize;

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
type Slot = u64;
type ValidatorIndex = u64;
type Depth = u64;

#[derive(Debug)]
pub enum CommitmentMapperTask {
    UpdateProofNode(Gindex, Slot),
    ProveZeroForDepth(Depth),
    UpdateValidatorProof(ValidatorIndex, Slot),
}

impl CommitmentMapperTask {
    pub fn log(&self) {
        match *self {
            CommitmentMapperTask::UpdateProofNode(gindex, slot) => println!(
                "{}",
                format!(
                    "Updating proof node at gindex {} for slot {}...",
                    gindex.to_string().magenta(),
                    slot.to_string().cyan()
                )
                .blue()
                .bold()
            ),
            CommitmentMapperTask::ProveZeroForDepth(depth) => {
                println!(
                    "{}",
                    format!("Proving zero for depth {}...", depth.to_string().magenta())
                        .blue()
                        .bold(),
                )
            }
            CommitmentMapperTask::UpdateValidatorProof(validator_index, slot) => {
                if validator_index != VALIDATOR_REGISTRY_LIMIT as u64 {
                    println!(
                        "{}",
                        format!(
                            "Updating validator proof at index {} for slot {}...",
                            validator_index.to_string().magenta(),
                            slot.to_string().cyan()
                        )
                        .blue()
                        .bold()
                    );
                } else {
                    println!("{}", "Proving zero validator...".blue().bold());
                }
            }
        };
    }
}

impl CommitmentMapperTask {
    pub fn deserialize(bytes: &[u8]) -> Option<CommitmentMapperTask> {
        match FromPrimitive::from_u8(u8::from_be_bytes(bytes[0..1].try_into().unwrap()))? {
            CommitmentMapperTaskType::UpdateProofNode => {
                let gindex = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                let slot = u64::from_be_bytes(bytes[9..17].try_into().unwrap());
                Some(CommitmentMapperTask::UpdateProofNode(gindex, slot))
            }
            CommitmentMapperTaskType::ProveZeroForDepth => {
                let depth = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Some(CommitmentMapperTask::ProveZeroForDepth(depth))
            }
            CommitmentMapperTaskType::UpdateValidatorProof => {
                let validator_index = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                let slot = u64::from_be_bytes(bytes[9..17].try_into().unwrap());
                Some(CommitmentMapperTask::UpdateValidatorProof(
                    validator_index,
                    slot,
                ))
            }
        }
    }
}
