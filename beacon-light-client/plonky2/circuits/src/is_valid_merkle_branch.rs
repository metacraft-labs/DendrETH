use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::utils::create_bool_target_array;
use crate::{sha256::make_circuits, utils::ETH_SHA256_BIT_SIZE};

pub struct IsValidMerkleBranchTargets {
    pub leaf: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub branch: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub index: Target,
    pub root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

pub type Sha256 = [BoolTarget; ETH_SHA256_BIT_SIZE];
pub type MerkleBranch<const DEPTH: usize> = [Sha256; DEPTH];

pub struct IsValidMerkleBranchTargetsResult {
    pub leaf: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub branch: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub index: Target,
    pub root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub is_valid: BoolTarget,
}

pub fn is_valid_merkle_branch_sha256_result<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    depth: usize,
) -> IsValidMerkleBranchTargetsResult {
    let index = builder.add_virtual_target();

    let leaf: [BoolTarget; ETH_SHA256_BIT_SIZE] = create_bool_target_array(builder);

    let branch: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]> = (0..depth)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let root: [BoolTarget; ETH_SHA256_BIT_SIZE] = create_bool_target_array(builder);

    let indexes = builder.split_le(index, depth + 1);

    let mut hashers = Vec::new();

    for i in 0..depth {
        hashers.push(make_circuits(builder, (2 * ETH_SHA256_BIT_SIZE) as u64));
        let current: [BoolTarget; ETH_SHA256_BIT_SIZE] = if i == 0 {
            leaf
        } else {
            hashers[i - 1].digest.clone().try_into().unwrap()
        };

        for j in 0..ETH_SHA256_BIT_SIZE {
            let el1 = builder._if(indexes[i], branch[i][j].target, current[j].target);
            builder.connect(hashers[i].message[j].target, el1);

            let el2 = builder._if(indexes[i], current[j].target, branch[i][j].target);
            builder.connect(hashers[i].message[j + ETH_SHA256_BIT_SIZE].target, el2);
        }
    }

    let mut is_valid = builder._true();

    for i in 0..ETH_SHA256_BIT_SIZE {
        let is_equal = builder.is_equal(hashers[depth - 1].digest[i].target, root[i].target);
        is_valid = builder.and(is_valid, is_equal);
    }

    IsValidMerkleBranchTargetsResult {
        leaf: leaf,
        branch: branch,
        index: index,
        root: root,
        is_valid,
    }
}

pub fn is_valid_merkle_branch_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    depth: usize,
) -> IsValidMerkleBranchTargets {
    let is_valid_merkle_branch_result_targets =
        is_valid_merkle_branch_sha256_result(builder, depth);

    let _true = builder._true();

    builder.connect(
        is_valid_merkle_branch_result_targets.is_valid.target,
        _true.target,
    );

    IsValidMerkleBranchTargets {
        leaf: is_valid_merkle_branch_result_targets.leaf,
        branch: is_valid_merkle_branch_result_targets.branch,
        index: is_valid_merkle_branch_result_targets.index,
        root: is_valid_merkle_branch_result_targets.root,
    }
}
