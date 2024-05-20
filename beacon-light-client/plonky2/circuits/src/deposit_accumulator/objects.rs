use plonky2::{hash::hash_types::HashOutTarget, iop::target::{BoolTarget, Target}, plonk::proof::ProofWithPublicInputsTarget};

use crate::{biguint::BigUintTarget, is_valid_merkle_branch::{MerkleBranch, Sha256}, is_valid_merkle_branch_poseidon::{IsValidMerkleBranchTargetsPoseidon, IsValidMerkleBranchTargetsPoseidonResult}, validator_hash_tree_root_poseidon::ValidatorPoseidonTargets};

use super::deposit_hash_tree_root_poseidon::DepositPoseidonTargets;

pub struct RangeObject {
    pub pubkey: [BoolTarget; 384],
    pub deposit_index: BigUintTarget,
    pub is_counted: BoolTarget,
    pub is_dummy: BoolTarget,
}

pub struct DepositAccumulatorOutputs {
    pub current_epoch: BigUintTarget,
    pub left_most: RangeObject,
    pub right_most: RangeObject,
}

pub struct DepositAccumulatorLeafTargets {
    pub validator: ValidatorPoseidonTargets,
    pub validator_deposit: DepositPoseidonTargets,
    pub commitment_mapper_proof: Vec<HashOutTarget>,
    pub validator_index: Target,
    pub validator_deposit_proof: Vec<HashOutTarget>,
    pub validator_deposit_index: Target,
    pub balance_tree_root: Sha256,
    pub balance_leaf: Sha256,
    pub balance_proof: MerkleBranch<22>,
    pub bls_signature_proof: ProofWithPublicInputsTarget<2>,
    pub is_dummy: BoolTarget,
    pub node: NodeTargets,
}

pub struct NodeTargets {
    pub leftmost: RangeObject,
    pub rightmost: RangeObject,
    pub accumulated: AccumulatedDataTargets,

    pub current_epoch: BigUintTarget,
    pub eth1_deposit_index: BigUintTarget,
    
    pub commitment_mapper_proof_root: HashOutTarget,
    pub merkle_tree_deposit_branch_root: HashOutTarget,
}

pub struct ValidatorStatsTargets {
    pub non_activated_validators_count: Target,
    pub active_validators_count: Target,
    pub exited_validators_count: Target,
    pub slashed_validators_count: Target,
}

pub struct AccumulatedDataTargets {
    pub balance_sum: BigUintTarget,
    pub deposits_count: Target,
    pub validator_stats: ValidatorStatsTargets,
}
