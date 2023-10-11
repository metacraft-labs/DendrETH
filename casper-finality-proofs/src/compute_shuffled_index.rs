use array_macro::array;
use plonky2::field::types::Field;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use sha2::{Digest, Sha256};
extern crate sha2;
use std::cmp;

use crate::frontend::builder::CircuitBuilder;
use crate::frontend::hash::sha::sha256::sha256;
use crate::frontend::vars::{
    ArrayVariable, BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitVariable,
    Variable,
};
use crate::prelude::PlonkParameters;

const SHUFFLE_ROUND_COUNT: usize = 90;

pub fn vanilla_compute_shuffled_index(index: &mut u64, index_count: u64, seed: [u8; 32]) -> u64 {
    assert!(*index < index_count);

    for current_round in 0..SHUFFLE_ROUND_COUNT {
        let mut bytes_current_round: [u8; 32] = [0; 32];
        bytes_current_round[0] = current_round as u8;

        let pivot = u64::from_be_bytes(
            Sha256::digest([seed, bytes_current_round].concat())[0..8]
                .try_into()
                .unwrap(),
        ) % index_count;
        let flip = (pivot + index_count - *index) % index_count;

        let position = cmp::max(flip, *index);
        let mut bytes_position: [u8; 32] = [0; 32];
        bytes_position[0] = position as u8;

        let source = Sha256::digest([seed, bytes_current_round, bytes_position].concat());

        let byte = (source[(position as usize % 256) / 8]) as u8;
        let bit = (byte >> (position as usize % 8)) % 2;

        if bit == 1 {
            *index = flip;
        }
    }

    *index
}

pub fn div_rem<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    lhs: Variable,
    rhs: Variable,
) -> Variable {
    let quotient = builder.div(lhs, rhs);
    let quotient_times_rhs = builder.mul(quotient, rhs);

    builder.sub(rhs, quotient_times_rhs)
}

impl<const N: usize> Default for BytesVariable<N> {
    fn default() -> BytesVariable<N> {
        unsafe { std::mem::zeroed() }
    }
}

impl<const N: usize> Default for ArrayVariable<ByteVariable, N> {
    fn default() -> ArrayVariable<ByteVariable, N> {
        unsafe { std::mem::zeroed() }
    }
}

fn compute_shuffled_index<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    index: Variable,
    index_count: Variable,
    seed: Bytes32Variable,
) -> Variable {
    const D: usize = 2;
    const CONST_32: usize = 32;
    type C = PoseidonGoldilocksConfig;
    type F = <C as GenericConfig<D>>::F;

    let index_lte_index_count = builder.lte(index, index_count);
    builder.assert_is_true(index_lte_index_count);

    let const_2 = builder.constant(L::Field::from_canonical_u8(2));
    let const_8 = builder.constant(L::Field::from_canonical_u8(8));
    let const_256 = builder.constant(L::Field::from_canonical_u16(256));
    for current_round in 0..SHUFFLE_ROUND_COUNT {
        let current_round_variable: Variable =
            builder.constant(L::Field::from_canonical_u8(current_round as u8));
        let current_round_bytes = current_round_variable.to_byte_variable(builder);

        let mut seed_round_to_be_hashed: BytesVariable<33> = BytesVariable::default();
        seed_round_to_be_hashed.0.copy_from_slice(&seed.0 .0);
        seed_round_to_be_hashed.0.to_vec().push(current_round_bytes);

        let seed_round_to_be_hashed_to_bits: [BoolTarget; 256] =
            array![BoolTarget::new_unsafe(Target::default()); 256];
        for i in 0..33 {
            for j in 0..8 {
                seed_round_to_be_hashed_to_bits
                    .to_vec()
                    .push(BoolTarget::new_unsafe(
                        seed_round_to_be_hashed.0[i].0[j].0 .0,
                    ));
            }
        }

        let seed_current_round_hashed =
            sha256(&mut builder.api, &seed_round_to_be_hashed_to_bits[..])
                .into_iter()
                .map(|x| BoolVariable(Variable(x.target)))
                .collect::<Vec<_>>();

        let seed_current_round_hash_to_variable = builder.le_sum(&seed_current_round_hashed[..64]);

        let pivot = div_rem(builder, seed_current_round_hash_to_variable, index_count);

        let sum_pivot_index_count = builder.add(pivot, index_count);
        let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);
        let flip = div_rem(builder, sum_pivot_index_count_sub_index, index_count);

        let index_lte_flip = builder.lte(index, flip);
        let position = builder.select(index_lte_flip, flip, index);

        let position_div_256 = builder.div(position, const_256);
        let position_div_256_bytes = position_div_256.to_byte_variable(builder);

        let mut source_to_be_hashed: BytesVariable<34> = BytesVariable::default();
        source_to_be_hashed.0.copy_from_slice(&seed.0 .0);
        source_to_be_hashed.0.to_vec().push(current_round_bytes);
        source_to_be_hashed.0.to_vec().push(position_div_256_bytes);

        let source_to_be_hashed_to_bits: [BoolTarget; 256] =
            array![BoolTarget::new_unsafe(Target::default()); 256];
        for i in 0..34 {
            for j in 0..8 {
                source_to_be_hashed_to_bits
                    .to_vec()
                    .push(BoolTarget::new_unsafe(source_to_be_hashed.0[i].0[j].0 .0));
            }
        }

        let source = sha256(&mut builder.api, &source_to_be_hashed_to_bits[..]);

        let mut source_in_bytes: ArrayVariable<ByteVariable, 34> = ArrayVariable::default();
        for i in 0..34 {
            let test = ByteVariable::from_variables_unsafe(
                &source_to_be_hashed_to_bits
                    .into_iter()
                    .map(|x| Variable(x.target))
                    .collect::<Vec<_>>()[i * 8..(i + 1) * 8],
            );
            source_in_bytes[i] = ByteVariable::from_variables_unsafe(
                &source_to_be_hashed_to_bits
                    .into_iter()
                    .map(|x| Variable(x.target))
                    .collect::<Vec<_>>()[i * 8..(i + 1) * 8],
            );
        }

        let position_mod_256 = div_rem(builder, position, const_8);
        let position_mod_256_div_8 = builder.div(position_mod_256, const_8);

        let source_result_index =
            builder.select_array(source_in_bytes.as_vec(), position_mod_256_div_8);
        let byte = source_result_index.to_variable(builder);

        let position_mod_8 = div_rem(builder, position, const_8);
        let byte_shl_position_mod_8 = builder.shl(byte, position_mod_8);
        let bit = div_rem(builder, byte_shl_position_mod_8, const_2);
        let bit_eq_1 = builder.is_equal(bit, builder.one());
        index = builder.select(bit_eq_1, flip, index);
    }

    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_shuffled_correctly() {
        let mut index = 18446744073709;
        let index_count = 1844674407370955161;
        let seed = [
            27, 26, 30, 6, 9, 28, 13, 0, 5, 8, 14, 12, 23, 21, 16, 4, 22, 31, 3, 10, 19, 11, 32,
            20, 7, 1, 25, 18, 17, 15, 2, 24,
        ];
        assert_eq!(
            vanilla_compute_shuffled_index(&mut index, index_count, seed),
            1520772600844238733
        );
    }

    #[test]
    #[should_panic]
    fn is_index_count_smaller() {
        let mut index = 1844674407370955161;
        let index_count = 18446744073709;
        let seed = [
            27, 26, 30, 6, 9, 28, 13, 0, 5, 8, 14, 12, 23, 21, 16, 4, 22, 31, 3, 10, 19, 11, 32,
            20, 7, 1, 25, 18, 17, 15, 2, 24,
        ];
        vanilla_compute_shuffled_index(&mut index, index_count, seed);
    }
}
