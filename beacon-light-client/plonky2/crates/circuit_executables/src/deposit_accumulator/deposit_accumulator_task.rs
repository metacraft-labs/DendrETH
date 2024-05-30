use crate::{commitment_mapper_context::CommitmentMapperContext, types::DepositAccumulatorTask};


pub fn handle_deposit_accumulator_task(task: DepositAccumulatorTask) {
    match task {
        DepositAccumulatorTask::ProveZeroForDepth(depth) => {
        }
        DepositAccumulatorTask::UpdateDepositProof(deposit_index, slot) => {
        }
        DepositAccumulatorTask::ZeroDeposit(deposit_index, slot) => {
        }
    }
    todo!()
}

async fn handle_update_validator_proof_task(
    ctx: &mut CommitmentMapperContext,
    validator_index: u64,
    slot: u64,
) -> Result<()> {
    todo!()
}

async fn handle_update_proof_node_task(
    ctx: &mut CommitmentMapperContext,
    gindex: u64,
    slot: u64,
) -> Result<()> {
    todo!()
}

async fn handle_prove_zero_for_depth_task(
    ctx: &mut CommitmentMapperContext,
    depth: u64,
) -> Result<()> {
    todo!()
}
