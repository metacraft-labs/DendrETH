use plonky2::{
    field::extension::Extendable,
    gadgets::hash,
    hash::{
        hash_types::{HashOutTarget, RichField},
        poseidon::PoseidonHash,
    },
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{
    utils::create_bool_target_array,
    validator_hash_tree_root::{hash_tree_root_validator_sha256, Validator},
    validator_hash_tree_root_poseidon::hash_tree_root_validator_poseidon,
};
pub struct ValidatorCommitment {
    pub validator: Validator,
    pub sha256_hash_tree_root: [BoolTarget; 256],
    pub poseidon_hash_tree_root: HashOutTarget,
}

pub fn validator_commitment<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
) -> ValidatorCommitment {
    let hash_tree_root_sha256 = hash_tree_root_validator_sha256(builder);

    let validator_poseidon = hash_tree_root_validator_poseidon(builder);

    // TODO: Map from Validator to ValidatorPoseidon and connect the mapping and the result

    ValidatorCommitment {
        validator: hash_tree_root_sha256.validator,
        sha256_hash_tree_root: hash_tree_root_sha256.hash_tree_root,
        poseidon_hash_tree_root: validator_poseidon.hash_tree_root,
    }
}
