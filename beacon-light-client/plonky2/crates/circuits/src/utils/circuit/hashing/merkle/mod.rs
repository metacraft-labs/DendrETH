use circuit::circuit_builder_extensions::CircuitBuilderExtensions;
use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::common_targets::Sha256Target;

pub mod poseidon;
pub mod sha256;

pub fn pick_left_and_right_hash<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    current_node: Sha256Target,
    sibling: Sha256Target,
    merkle_path_bit: BoolTarget,
) -> (Sha256Target, Sha256Target) {
    let left = builder.select_target(merkle_path_bit, &sibling, &current_node);
    let right = builder.select_target(merkle_path_bit, &current_node, &sibling);

    (left, right)
}
