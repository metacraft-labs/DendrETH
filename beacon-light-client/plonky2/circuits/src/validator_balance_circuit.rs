use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    hash_tree_root::hash_tree_root,
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    utils::create_bool_target_array,
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidon, ValidatorPoseidonHashTreeRootTargets,
    },
};

pub struct ValidatorBalanceVerificationTargets {
    pub range_total_value: Target,
    pub range_balances_root: [BoolTarget; 256],
    pub range_validator_commitment: HashOutTarget,
    pub validators: Vec<ValidatorPoseidon>,
    pub balances: Vec<[BoolTarget; 256]>,
    pub withdrawal_credentials: [Target; 5],
}

pub fn validator_balance_verification<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    validators_len: usize,
) -> ValidatorBalanceVerificationTargets {
    if !validators_len.is_power_of_two() {
        panic!("validators_len must be a power of two");
    }

    let balances_len = validators_len / 4;

    let balances_leaves: Vec<[BoolTarget; 256]> = (0..balances_len)
        .map(|_| create_bool_target_array(builder))
        .collect();

    let balances_hash_tree_root_targets = hash_tree_root(builder, balances_len);

    for i in 0..balances_len {
        for j in 0..256 {
            builder.connect(
                balances_hash_tree_root_targets.leaves[i][j].target,
                balances_leaves[i][j].target,
            );
        }
    }

    let validators_leaves: Vec<ValidatorPoseidonHashTreeRootTargets> = (0..validators_len)
        .map(|_| hash_tree_root_validator_poseidon(builder))
        .collect();

    let hash_tree_root_poseidon_targets = hash_tree_root_poseidon(builder, validators_len);

    for i in 0..validators_len {
        builder.connect_hashes(
            hash_tree_root_poseidon_targets.leaves[i],
            validators_leaves[i].hash_tree_root,
        );
    }

    let withdrawal_credentials = [
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
    ];

    let mut sums: Vec<Target> = Vec::new();

    sums.push(builder.zero());

    for i in 0..validators_len {
        let mut all_equal = Vec::new();
        all_equal.push(builder._true());

        for j in 0..5 {
            let is_equal = builder.is_equal(
                validators_leaves[i].validator.withdrawal_credentials[j],
                withdrawal_credentials[j],
            );

            all_equal.push(builder.and(all_equal[j], is_equal));
        }

        // the balance shouldn't be more than 63 bits anyway
        let bits = &balances_leaves[i / 4][((i % 4) * 64)..(((i % 4) * 64) + 63)];

        let reversed_bits = bits.chunks(8).map(|chunk| chunk.iter().rev()).flatten();

        let balance_sum = builder.le_sum(reversed_bits);
        let zero = builder.zero();
        let current = builder._if(all_equal[5], balance_sum, zero);

        sums.push(builder.add(sums[i], current));
    }

    ValidatorBalanceVerificationTargets {
        range_total_value: sums[validators_len],
        range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
        range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
        validators: validators_leaves.iter().map(|v| v.validator).collect(),
        balances: balances_leaves,
        withdrawal_credentials: withdrawal_credentials,
    }
}
