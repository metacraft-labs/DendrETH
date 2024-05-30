use num_derive::FromPrimitive;

type Gindex = u64;
type Slot = u64;
type ValidatorIndex = u64;
type Depth = u64;
type DepositIndex = u64;

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum CommitmentMapperTaskType {
    UpdateProofNode,
    ProveZeroForDepth,
    UpdateValidatorProof,
    ZeroOutValidator,
}

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum DepositAccumulatorTaskType {
    ProveZeroForDepth,
    DepositProof,
    DepositNodeDeposit,
}

#[derive(Debug)]
pub enum CommitmentMapperTask {
    UpdateProofNode(Gindex, Slot),
    ProveZeroForDepth(Depth),
    UpdateValidatorProof(ValidatorIndex, Slot),
    ZeroOutValidator(ValidatorIndex, Slot),
}

#[derive(Debug)]
pub enum DepositAccumulatorTask {
    ProveZeroForDepth(Depth),
    UpdateDepositProof(DepositIndex, Slot),
    ZeroDeposit(DepositIndex, Slot),
}
