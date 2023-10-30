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
                "{}",
                format!(
                    "Updating proof node at gindex {} for epoch {}...",
                    gindex.to_string().magenta(),
                    epoch.to_string().cyan()
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
            CommitmentMapperTask::UpdateValidatorProof(validator_index, epoch) => {
                if validator_index != VALIDATOR_REGISTRY_LIMIT as u64 {
                    println!(
                        "{}",
                        format!(
                            "Updating validator proof at index {} for epoch {}...",
                            validator_index.to_string().magenta(),
                            epoch.to_string().cyan()
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
}

#[derive(FromPrimitive)]
#[repr(u8)]
enum CommitmentMapperAccumulatorTaskType {
    AppendValidatorAccumulatorProof,
    UpdateProofNode,
    ProveZeroForDepth,
}

#[derive(Debug)]
pub enum CommitmentMapperAccumulatorTask {
    AppendValidatorAccumulatorProof(Gindex),
    UpdateProofNode(Gindex),
    ProveZeroForDepth(Depth),
}

impl CommitmentMapperAccumulatorTask {
    pub fn log(&self) {
        match *self {
            CommitmentMapperAccumulatorTask::AppendValidatorAccumulatorProof(gindex) => println!(
                "{}",
                format!(
                    "Appending validator accumulator proof at index {}...",
                    gindex.to_string().magenta()
                )
                .blue()
                .bold()
            ),
            CommitmentMapperAccumulatorTask::UpdateProofNode(gindex) => println!(
                "{}",
                format!(
                    "Updating proof node at gindex {}...",
                    gindex.to_string().magenta()
                )
                .blue()
                .bold()
            ),
            CommitmentMapperAccumulatorTask::ProveZeroForDepth(depth) => {
                println!(
                    "{}",
                    format!("Proving zero for depth {}...", depth.to_string().magenta())
                        .blue()
                        .bold(),
                )
            }
        };
    }
}

impl CommitmentMapperAccumulatorTask {
    pub fn deserialize(bytes: &[u8]) -> Option<CommitmentMapperAccumulatorTask> {
        match FromPrimitive::from_u8(u8::from_be_bytes(bytes[0..1].try_into().unwrap()))? {
            CommitmentMapperAccumulatorTaskType::AppendValidatorAccumulatorProof => {
                let gindex = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Some(CommitmentMapperAccumulatorTask::AppendValidatorAccumulatorProof(gindex))
            }
            CommitmentMapperAccumulatorTaskType::UpdateProofNode => {
                let gindex = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Some(CommitmentMapperAccumulatorTask::UpdateProofNode(gindex))
            }
            CommitmentMapperAccumulatorTaskType::ProveZeroForDepth => {
                let depth = u64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Some(CommitmentMapperAccumulatorTask::ProveZeroForDepth(depth))
            }
        }
    }
}
