use circuit::ToTargets;
use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use plonky2_crypto::biguint::BigUintTarget;

use crate::{
    common_targets::{PoseidonMerkleBranchTarget, ValidatorTarget},
    utils::circuit::{
        biguint_to_le_bits_target,
        hashing::poseidon::{poseidon, poseidon_pair},
    },
};

pub fn hash_tree_root_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves: &[HashOutTarget],
) -> HashOutTarget {
    assert!(leaves.len().is_power_of_two());

    let mut level = leaves.to_owned();

    while level.len() != 1 {
        level = level
            .iter()
            .tuples()
            .map(|(&left, &right)| poseidon_pair(builder, left, right))
            .collect_vec();
    }

    level[0]
}

pub fn hash_validator_poseidon_or_zeroes<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
    condition: BoolTarget,
) -> HashOutTarget {
    let validator_hash = hash_validator_poseidon(builder, validator);
    HashOutTarget {
        elements: validator_hash
            .elements
            .map(|element| builder.mul(condition.target, element)),
    }
}

pub fn hash_validator_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
) -> HashOutTarget {
    let leaves = &[
        poseidon(builder, validator.pubkey.to_targets()),
        poseidon(builder, validator.withdrawal_credentials.to_targets()),
        poseidon(builder, validator.effective_balance.to_targets()),
        poseidon(builder, validator.slashed.to_targets()),
        poseidon(builder, validator.activation_eligibility_epoch.to_targets()),
        poseidon(builder, validator.activation_epoch.to_targets()),
        poseidon(builder, validator.exit_epoch.to_targets()),
        poseidon(builder, validator.withdrawable_epoch.to_targets()),
    ];

    hash_tree_root_poseidon(builder, leaves)
}

pub fn hash_outs_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &HashOutTarget,
    right: &HashOutTarget,
) -> BoolTarget {
    let mut is_equal = builder._true();

    for i in 0..left.elements.len() {
        let is_equal_element = builder.is_equal(left.elements[i], right.elements[i]);
        is_equal = builder.and(is_equal, is_equal_element);
    }

    is_equal
}

pub fn pick_left_and_right_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_node: HashOutTarget,
    sibling: HashOutTarget,
    merkle_path_bit: BoolTarget,
) -> (HashOutTarget, HashOutTarget) {
    let mut left = Vec::new();
    let mut right = Vec::new();

    for idx in 0..NUM_HASH_OUT_ELTS {
        left.push(builder._if(
            merkle_path_bit,
            sibling.elements[idx],
            current_node.elements[idx],
        ));

        right.push(builder._if(
            merkle_path_bit,
            current_node.elements[idx],
            sibling.elements[idx],
        ));
    }

    (
        HashOutTarget {
            elements: left.try_into().unwrap(),
        },
        HashOutTarget {
            elements: right.try_into().unwrap(),
        },
    )
}

pub fn restore_merkle_root_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> HashOutTarget {
    let bits = biguint_to_le_bits_target(builder, &gindex);
    let mut current = leaf.clone();

    for level in 0..DEPTH {
        let (left_hash, right_hash) =
            pick_left_and_right_hash(builder, current, branch[level], bits[level]);
        current = poseidon_pair(builder, left_hash, right_hash);
    }

    current
}

pub fn validate_merkle_proof_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    root: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> BoolTarget {
    let restored_root = restore_merkle_root_poseidon(builder, leaf, branch, gindex);
    hash_outs_are_equal(builder, &restored_root, root)
}

pub fn assert_merkle_proof_is_valid_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    root: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) {
    let is_valid = validate_merkle_proof_poseidon(builder, leaf, root, branch, gindex);
    let _true = builder._true();
    builder.connect(is_valid.target, _true.target);
}

pub fn hash_outs_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    left: &HashOutTarget,
    right: &HashOutTarget,
) -> BoolTarget {
    let mut is_equal = builder._true();

    for i in 0..left.elements.len() {
        let is_equal_element = builder.is_equal(left.elements[i], right.elements[i]);
        is_equal = builder.and(is_equal, is_equal_element);
    }

    is_equal
}

pub fn pick_left_and_right_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_node: HashOutTarget,
    sibling: HashOutTarget,
    merkle_path_bit: BoolTarget,
) -> (HashOutTarget, HashOutTarget) {
    let mut left = Vec::new();
    let mut right = Vec::new();

    for idx in 0..NUM_HASH_OUT_ELTS {
        left.push(builder._if(
            merkle_path_bit,
            sibling.elements[idx],
            current_node.elements[idx],
        ));

        right.push(builder._if(
            merkle_path_bit,
            current_node.elements[idx],
            sibling.elements[idx],
        ));
    }

    (
        HashOutTarget {
            elements: left.try_into().unwrap(),
        },
        HashOutTarget {
            elements: right.try_into().unwrap(),
        },
    )
}

pub fn restore_merkle_root_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> HashOutTarget {
    let bits = biguint_to_le_bits_target::<F, D, 2>(builder, &gindex);
    let mut current = leaf.clone();

    for level in 0..DEPTH {
        let (left_hash, right_hash) =
            pick_left_and_right_hash(builder, current, branch[level], bits[level]);
        current = poseidon_pair(builder, left_hash, right_hash);
    }

    current
}

pub fn validate_merkle_proof_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    root: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) -> BoolTarget {
    let restored_root = restore_merkle_root_poseidon(builder, leaf, branch, gindex);
    hash_outs_are_equal(builder, &restored_root, root)
}

pub fn assert_merkle_proof_is_valid_poseidon<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &HashOutTarget,
    root: &HashOutTarget,
    branch: &PoseidonMerkleBranchTarget<DEPTH>,
    gindex: &BigUintTarget,
) {
    let is_valid = validate_merkle_proof_poseidon(builder, leaf, root, branch, gindex);
    let _true = builder._true();
    builder.connect(is_valid.target, _true.target);
}
