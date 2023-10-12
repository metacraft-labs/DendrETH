use plonky2::field::types::Field;
use plonky2x::prelude::{BoolVariable, Bytes32Variable, CircuitBuilder, PlonkParameters, BytesVariable, Variable};
use crate::utils::variable::{to_bits, to_byte_variable};
use crate::utils::universal::{assert_is_true, le_sum, div_rem, exp_from_bits};

fn compute_shuffled_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    mut index: Variable,
    index_count: Variable,
    seed: Bytes32Variable,
) -> Variable {
    let index_lte_index_count = builder.lte(index, index_count);
    assert_is_true(builder, index_lte_index_count);

    let const_1: Variable = builder.constant(L::Field::from_canonical_u8(1));
    let const_2: Variable = builder.constant(L::Field::from_canonical_u8(2));
    let const_8 = builder.constant(L::Field::from_canonical_u8(8));
    let const_256 = builder.constant(L::Field::from_canonical_u16(256));
    const SHUFFLE_ROUND_COUNT: usize = 90;
    for current_round in 0..SHUFFLE_ROUND_COUNT {
        let current_round_variable: Variable =
            builder.constant(L::Field::from_canonical_u8(current_round as u8));
        let current_round_bytes = to_byte_variable(current_round_variable, builder);

        let mut seed_round_to_be_hashed: BytesVariable<33> = unsafe { std::mem::zeroed() };
        seed_round_to_be_hashed.0.copy_from_slice(&seed.0 .0);
        seed_round_to_be_hashed.0.to_vec().push(current_round_bytes);

        let seed_current_round_hashed = builder.sha256(&seed_round_to_be_hashed.0);

        let mut seed_current_round_hashed_first_64_bits: [BoolVariable; 64] =
            [unsafe { std::mem::zeroed() }; 64];
        for i in 0..8 {
            for j in 0..8 {
                seed_current_round_hashed_first_64_bits[i] = seed_current_round_hashed.0 .0[i].0[j];
            }
        }

        let seed_current_round_hash_to_variable = le_sum(builder, &seed_current_round_hashed_first_64_bits);

        let pivot = div_rem(builder, seed_current_round_hash_to_variable, index_count);

        let sum_pivot_index_count = builder.add(pivot, index_count);
        let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
        let flip = div_rem(builder, sum_pivot_index_count_sub_index, index_count);

        let index_lte_flip = builder.lte(index, flip);
        let position = builder.select(index_lte_flip, flip, index);

        let position_div_256 = builder.div(position, const_256);
        let position_div_256_bytes = to_byte_variable(position_div_256, builder);

        let mut source_to_be_hashed: BytesVariable<34> = unsafe { std::mem::zeroed() };
        source_to_be_hashed.0.copy_from_slice(&seed.0 .0);
        source_to_be_hashed.0.to_vec().push(current_round_bytes);
        source_to_be_hashed.0.to_vec().push(position_div_256_bytes);

        let source = builder.sha256(&source_to_be_hashed.0);

        let position_mod_256 = div_rem(builder, position, const_8);
        let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

        let byte = builder.select_array(&source.0 .0, position_mod_256_div_8);
        let byte_to_variable = byte.to_variable(builder);

        let position_mod_8 = div_rem(builder, position, const_8);
        let position_mod_8_to_bits: [BoolVariable; 8] = to_bits(position_mod_8, builder);
        let const_2_pow_position_mod_8 = exp_from_bits(builder, const_2, &position_mod_8_to_bits);

        let byte_shl_position_mod_8 =
            builder.div(byte_to_variable, const_2_pow_position_mod_8);
        let bit = div_rem(builder, byte_shl_position_mod_8, const_2);
        let bit_eq_1 = builder.is_equal(bit, const_1);
        index = builder.select(bit_eq_1, flip, index);
    }

    index
}

#[cfg(test)]
mod tests {}
