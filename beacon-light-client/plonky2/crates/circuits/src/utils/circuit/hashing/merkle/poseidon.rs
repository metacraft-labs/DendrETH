use itertools::Itertools;
use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{common_targets::ValidatorTarget, utils::circuit::hashing::poseidon::poseidon_pair};

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
    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.pubkey.iter().map(|x| x.target).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawal_credentials
                .iter()
                .map(|x| x.target)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .effective_balance
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![validator.slashed.target]),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_eligibility_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .activation_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator.exit_epoch.limbs.iter().map(|x| x.0).collect(),
        ),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(
            validator
                .withdrawable_epoch
                .limbs
                .iter()
                .map(|x| x.0)
                .collect(),
        ),
    ];

    hash_tree_root_poseidon(builder, &leaves)
}
