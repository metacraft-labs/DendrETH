use plonky2::plonk::proof::ProofWithPublicInputsTarget;

pub type DepositAccumulatorProofTarget = ProofWithPublicInputsTarget<2>;
pub type ValidatorBalanceProofTargets<const N: usize> = ProofWithPublicInputsTarget<2>;
