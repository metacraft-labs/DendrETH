use plonky2::{
    field::extension::Extendable,
    hash::{hash_types::{HashOutTarget, RichField}, poseidon::PoseidonHash},
    iop::target::Target,
    plonk::circuit_builder::CircuitBuilder,
};

pub struct ValidatorPoseidon {
    pub pubkey: [Target; 7],
    pub activation_epoch: [Target; 2],
    pub exit_epoch: [Target; 2],
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
        activation_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
        exit_epoch: [builder.add_virtual_target(), builder.add_virtual_target()],
    };

    let hash_tree_root = builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![
        validator.pubkey[0],
        validator.pubkey[1],
        validator.pubkey[2],
        validator.pubkey[3],
        validator.pubkey[4],
        validator.pubkey[5],
        validator.pubkey[6],
        validator.activation_epoch[0],
        validator.activation_epoch[1],
        validator.exit_epoch[0],
        validator.exit_epoch[1],
    ]);

    ValidatorPoseidonHashTreeRootTargets {
        validator,
        hash_tree_root,
    }
}
