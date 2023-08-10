
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    is_active_validator::is_active_validator,
    hash_tree_root::hash_tree_root,
    hash_tree_root_poseidon::hash_tree_root_poseidon,
    utils::{create_bool_target_array, to_mixed_endian},
    validator_hash_tree_root_poseidon::{
        hash_tree_root_validator_poseidon, ValidatorPoseidon, ValidatorPoseidonHashTreeRootTargets,
    },
};

pub struct ValidatorBalanceVerificationTargets {
    pub range_total_value: Target,
    pub range_balances_root: [BoolTarget; 256],
    pub range_validator_commitment: HashOutTarget,
    pub validators: Vec<ValidatorPoseidon>,
    pub validator_is_zero: Vec<BoolTarget>,
    pub balances: Vec<[BoolTarget; 256]>,
    pub withdrawal_credentials: [Target; 5],
    pub current_epoch: [Target; 2],
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

    let validator_is_zero: Vec<BoolTarget> = (0..validators_len)
        .map(|_| builder.add_virtual_bool_target_safe())
        .collect();

    let zero_hash = builder.zero();

    for i in 0..validators_len {
        let mut elements = [zero_hash; 4];

        for (j, _) in validators_leaves[i]
            .hash_tree_root
            .elements
            .iter()
            .enumerate()
        {
            elements[j] = builder._if(
                validator_is_zero[i],
                zero_hash,
                validators_leaves[i].hash_tree_root.elements[j],
            );
        }

        builder.connect_hashes(
            hash_tree_root_poseidon_targets.leaves[i],
            HashOutTarget { elements },
        );
    }

    let withdrawal_credentials = [
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
        builder.add_virtual_target(),
    ];

    let current_epoch = [builder.add_virtual_target(), builder.add_virtual_target()];

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

        let reversed_bits = to_mixed_endian(bits);

        let balance_sum = builder.le_sum(reversed_bits);
        let zero = builder.zero();

        let range_check_targets = is_active_validator(builder);

        builder.connect(
            range_check_targets.activation_epoch[0],
            validators_leaves[i].validator.activation_epoch[0],
        );
        builder.connect(
            range_check_targets.activation_epoch[1],
            validators_leaves[i].validator.activation_epoch[1],
        );
        builder.connect(
            range_check_targets.exit_epoch[0],
            validators_leaves[i].validator.exit_epoch[0],
        );
        builder.connect(
            range_check_targets.exit_epoch[1],
            validators_leaves[i].validator.exit_epoch[1],
        );

        builder.connect(range_check_targets.current_epoch[0], current_epoch[0]);
        builder.connect(range_check_targets.current_epoch[1], current_epoch[1]);

        let is_valid = builder.and(all_equal[5], range_check_targets.result);

        let current = builder._if(is_valid, balance_sum, zero);

        sums.push(builder.add(sums[i], current));
    }

    ValidatorBalanceVerificationTargets {
        validator_is_zero: validator_is_zero,
        range_total_value: sums[validators_len],
        range_balances_root: balances_hash_tree_root_targets.hash_tree_root,
        range_validator_commitment: hash_tree_root_poseidon_targets.hash_tree_root,
        validators: validators_leaves.iter().map(|v| v.validator).collect(),
        balances: balances_leaves,
        withdrawal_credentials: withdrawal_credentials,
        current_epoch,
    }
}
