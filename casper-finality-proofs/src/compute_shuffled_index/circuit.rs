use crate::utils::plonky2x_extensions::{assert_is_true, max};
use plonky2x::prelude::{Bytes32Variable, CircuitBuilder, PlonkParameters, U64Variable};

use super::helpers::{compute_bit, compute_byte, compute_flip, compute_pivot, compute_source};

pub fn define<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    shuffle_round_count: u8,
) {
    let mut index = builder.read::<U64Variable>();
    let index_count = builder.read::<U64Variable>();
    let seed = builder.read::<Bytes32Variable>();

    let index_lt_index_count = builder.lt(index, index_count);
    assert_is_true(builder, index_lt_index_count);

    for current_round in 0..shuffle_round_count {
        let pivot = compute_pivot(builder, seed, index_count, current_round);
        let flip = compute_flip(builder, pivot, index_count, index);

        let position = max(builder, index, flip);
        let source = compute_source(builder, position, seed, current_round);

        let byte = compute_byte(builder, source, position);
        let bit = compute_bit(builder, byte, position);

        index = builder.select(bit, flip, index);
    }

    builder.write::<U64Variable>(index);
}
