use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions, targets::uint::Uint64Target,
    SSZHashTreeRoot,
};
use itertools::Itertools;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    common_targets::{
        MerklelizedValidatorTarget, SSZTarget, Sha256MerkleBranchTarget, Sha256Target,
        ValidatorTarget,
    },
    utils::circuit::{bool_arrays_are_equal, hashing::sha256::sha256_pair},
};

use super::pick_left_and_right_hash;

pub fn restore_merkle_root_sha256<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: Uint64Target,
) -> Sha256Target {
    let bits = gindex.to_le_bits(builder);
    let mut current = leaf.clone();

    for level in 0..DEPTH {
        let (left_hash, right_hash) =
            pick_left_and_right_hash(builder, current, branch[level], bits[level]);
        current = sha256_pair(builder, left_hash.as_slice(), right_hash.as_slice());
    }

    current
}

pub fn validate_merkle_proof_sha256<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: Uint64Target,
) -> BoolTarget {
    let restored_root = restore_merkle_root_sha256(builder, leaf, branch, gindex);
    bool_arrays_are_equal(builder, root, &restored_root)
}

pub fn assert_merkle_proof_is_valid_sha256<
    const DEPTH: usize,
    F: RichField + Extendable<D>,
    const D: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
    leaf: &Sha256Target,
    root: &Sha256Target,
    branch: &Sha256MerkleBranchTarget<DEPTH>,
    gindex: Uint64Target,
) {
    let is_valid = validate_merkle_proof_sha256(builder, leaf, root, branch, gindex);
    builder.assert_true(is_valid);
}

pub fn validate_merkle_proof_const_sha256<
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
    let gindex_target = Uint64Target::constant(gindex, builder);
    let restored_root = restore_merkle_root_sha256(builder, leaf, branch, gindex_target);
    bool_arrays_are_equal(builder, root, &restored_root)
}

pub fn assert_merkle_proof_is_valid_const_sha256<
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
    let is_valid = validate_merkle_proof_const_sha256(builder, leaf, root, branch, gindex);
    builder.assert_true(is_valid);
}

pub fn hash_tree_root_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    leaves: &[SSZTarget],
) -> Sha256Target {
    assert!(leaves.len().is_power_of_two());

    let mut level = leaves.to_owned();

    while level.len() != 1 {
        level = level
            .iter()
            .tuples()
            .map(|(left, right)| sha256_pair(builder, left, right))
            .collect_vec();
    }

    level[0]
}

pub fn hash_validator_sha256<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
) -> Sha256Target {
    let leaves = vec![
        sha256_pair(builder, &validator.pubkey[0], &validator.pubkey[1]),
        validator.withdrawal_credentials,
        validator.effective_balance,
        validator.slashed,
        validator.activation_eligibility_epoch,
        validator.activation_epoch,
        validator.exit_epoch,
        validator.withdrawable_epoch,
    ];

    hash_tree_root_sha256(builder, &leaves)
}

pub fn hash_validator_sha256_or_zeroes<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &MerklelizedValidatorTarget,
    condition: BoolTarget,
) -> Sha256Target {
    let validator_hash = hash_validator_sha256(builder, validator);
    validator_hash.map(|bit| builder.and(condition, bit))
}

pub fn merklelize_validator_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validator: &ValidatorTarget,
) -> MerklelizedValidatorTarget {
    let zero_bits_128 = [BoolTarget::new_unsafe(builder.zero()); 128];

    let first_pubkey_leaf: SSZTarget = (&validator.pubkey[0..256]).try_into().unwrap();
    let second_pubkey_leaf: SSZTarget = [&validator.pubkey[256..], &zero_bits_128[..]]
        .concat()
        .try_into()
        .unwrap();

    MerklelizedValidatorTarget {
        pubkey: [first_pubkey_leaf, second_pubkey_leaf],
        withdrawal_credentials: validator.withdrawal_credentials,
        effective_balance: validator.effective_balance.ssz_hash_tree_root(builder),
        slashed: validator.slashed.ssz_hash_tree_root(builder),
        activation_eligibility_epoch: validator
            .activation_eligibility_epoch
            .ssz_hash_tree_root(builder),
        activation_epoch: validator.activation_epoch.ssz_hash_tree_root(builder),
        exit_epoch: validator.exit_epoch.ssz_hash_tree_root(builder),
        withdrawable_epoch: validator.withdrawable_epoch.ssz_hash_tree_root(builder),
    }
}
