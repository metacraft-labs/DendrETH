use num_bigint::BigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::{
    common_targets::{Sha256MerkleBranchTarget, Sha256Target},
    utils::utils::{biguint_to_le_bits_target, create_bool_target_array, ETH_SHA256_BIT_SIZE},
};

use super::sha256::{bool_arrays_are_equal, make_circuits, sha256_pair};

pub struct IsValidMerkleBranchTargets {
    pub leaf: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub branch: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub index: Target,
    pub root: [BoolTarget; ETH_SHA256_BIT_SIZE],
}

pub struct IsValidMerkleBranchTargetsResult {
    pub leaf: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub branch: Vec<[BoolTarget; ETH_SHA256_BIT_SIZE]>,
    pub index: Target,
    pub root: [BoolTarget; ETH_SHA256_BIT_SIZE],
    pub is_valid: BoolTarget,
}

pub fn pick_left_and_right_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_node: Sha256Target,
    sibling: Sha256Target,
    merkle_path_bit: BoolTarget,
) -> (Sha256Target, Sha256Target) {
    let mut left = Vec::new();
    let mut right = Vec::new();

    for idx in 0..ETH_SHA256_BIT_SIZE {
        left.push(BoolTarget::new_unsafe(builder._if(
            merkle_path_bit,
            sibling[idx].target,
            current_node[idx].target,
        )));
        right.push(BoolTarget::new_unsafe(builder._if(
            merkle_path_bit,
            current_node[idx].target,
            sibling[idx].target,
        )));
    }
    (left.try_into().unwrap(), right.try_into().unwrap())
}

pub fn restore_merkle_root<const DEPTH: usize, F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> Sha256Target {
    let bits = biguint_to_le_bits_target::<F, D, 2>(builder, &gindex);
    let mut current = leaf.clone();

    for level in 0..DEPTH {
        let (left_hash, right_hash) =
            pick_left_and_right_hash(builder, current, branch[level], bits[level]);
        current = sha256_pair(builder, left_hash.as_slice(), right_hash.as_slice());
    }

    current
}

pub fn validate_merkle_proof<const DEPTH: usize, F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> BoolTarget {
    let restored_root = restore_merkle_root(builder, leaf, branch, gindex);
    bool_arrays_are_equal(builder, root, &restored_root)
}

pub fn assert_merkle_proof_is_valid<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) {
    let is_valid = validate_merkle_proof(builder, leaf, root, branch, gindex);
    let _true = builder._true();
    builder.connect(is_valid.target, _true.target);
}

pub fn validate_merkle_proof_const<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: u64,
) -> BoolTarget {
    let gindex_target = builder.constant_biguint(&BigUint::from(gindex));
    let restored_root = restore_merkle_root(builder, leaf, branch, &gindex_target);
    bool_arrays_are_equal(builder, root, &restored_root)
}

pub fn assert_merkle_proof_is_valid_const<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: u64,
) {
    let is_valid = validate_merkle_proof_const(builder, leaf, root, branch, gindex);
    let _true = builder._true();
    builder.connect(is_valid.target, _true.target);
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
        leaf,
        branch,
        index,
        root,
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
