use circuit::ToTargets;
use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    common_targets::ValidatorTarget,
    utils::circuit::hashing::poseidon::{poseidon, poseidon_pair},
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
