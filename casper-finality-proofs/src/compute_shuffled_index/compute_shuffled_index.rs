use crate::utils::utils::assert_is_true;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        BoolVariable, Bytes32Variable, CircuitBuilder, PlonkParameters, U64Variable, CircuitVariable
    },
};

use super::compute_shuffled_index_helpers::{compute_pivot, compute_source, compute_byte, compute_bit, compute_hash};

#[derive(Debug, Clone)]
pub struct ComputeShuffledIndex;

impl Circuit for ComputeShuffledIndex {
    fn define<L: PlonkParameters<D>, const D: usize>(
        builder: &mut CircuitBuilder<L, D>
    ) {
        let mut index = builder.read::<U64Variable>();
        let index_count = builder.read::<U64Variable>();
        let seed = builder.read::<Bytes32Variable>();

        let index_lte_index_count = builder.lte(index, index_count);
        assert_is_true(builder, index_lte_index_count);

        const SHUFFLE_ROUND_COUNT: usize = 90;

        for current_round in 0..SHUFFLE_ROUND_COUNT {
            let hash = compute_hash(builder, seed, current_round);
            let pivot = compute_pivot(builder, hash, index_count);

            let sum_pivot_index_count = builder.add(pivot, index_count);
            let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
            let flip = builder.rem(sum_pivot_index_count_sub_index, index_count);

            let index_lte_flip = builder.lte(index, flip);
            let position = builder.select(index_lte_flip, flip, index);
            let source = compute_source(builder, position, seed, current_round);

            let byte = compute_byte(builder, source, position);
            let bit = compute_bit(builder, byte, position);

            index = builder.select(BoolVariable(bit.variables()[0]), flip, index);
        }

        builder.write::<U64Variable>(index);
    }
}
