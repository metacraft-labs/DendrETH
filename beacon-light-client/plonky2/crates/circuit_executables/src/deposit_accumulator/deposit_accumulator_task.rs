
#[derive(FromPrimitive)]
#[repr(u8)]
enum DepositAccumulatorTaskType {
    ProveZeroForDepth,
    UpdateDepositProof,
    ZeroDeposit,
}


type DepositIndex = u64;

#[derive(Debug)]
pub enum DepositAccumulatorTask {
    ProveZeroForDepth(Depth),
    UpdateDepositProof(DepositIndex, Slot),
    ZeroDeposit(DepositIndex, Slot),
}

