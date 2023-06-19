use plonky2::{
    field::extension::Extendable,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::Target,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::hash_tree_root_poseidon::hash_tree_root_poseidon;

pub struct ValidatorPoseidon {
    pub pubkey: [Target; 7],
    pub withdrawal_credentials: [Target; 5],
    pub effective_balance: [Target; 2],
    pub slashed: [Target; 1],
    pub activation_eligibility_epoch: [Target; 2],
    pub activation_epoch: [Target; 2],
    pub exit_epoch: [Target; 2],
    pub withdrawable_epoch: [Target; 2],
}

pub struct ValidatorPoseidonHashTreeRootTargets {
    pub validator: ValidatorPoseidon,
    pub hash_tree_root: HashOutTarget,
}

pub fn hash_tree_root_validator_poseidon<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorPoseidonHashTreeRootTargets {
    let validator = ValidatorPoseidon {
        pubkey: [
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
        ],
        withdrawal_credentials: [
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
            builder.add_virtual_target(),
        ],
        effective_balance: [builder.add_virtual_target(), builder.add_virtual_target()],
        slashed: [builder.add_virtual_target()],
        activation_eligibility_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
        activation_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
        exit_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
        withdrawable_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
    };

    let leaves = vec![
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.pubkey.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.withdrawal_credentials.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.effective_balance.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.slashed.to_vec()),
        builder
            .hash_n_to_hash_no_pad::<PoseidonHash>(validator.activation_eligibility_epoch.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.activation_epoch.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.exit_epoch.to_vec()),
        builder.hash_n_to_hash_no_pad::<PoseidonHash>(validator.withdrawable_epoch.to_vec()),
    ];

    let hash_tree_root_poseidon = hash_tree_root_poseidon(builder, leaves.len());

    for i in 0..leaves.len() {
        builder.connect_hashes(leaves[i], hash_tree_root_poseidon.leaves[i]);
    }

    ValidatorPoseidonHashTreeRootTargets {
        validator,
        hash_tree_root: hash_tree_root_poseidon.hash_tree_root,
    }
}
