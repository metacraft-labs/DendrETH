use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::common_targets::Sha256Target;

pub mod poseidon;
pub mod sha256;
pub mod ssz;

pub fn pick_left_and_right_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_node: Sha256Target,
    sibling: Sha256Target,
    merkle_path_bit: BoolTarget,
) -> (Sha256Target, Sha256Target) {
    let mut left = Vec::new();
    let mut right = Vec::new();

    for idx in 0..256 {
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
