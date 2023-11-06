use itertools::Itertools;
use plonky2::field::types::Field;
use plonky2x::{
    frontend::vars::EvmVariable,
    prelude::{
        BoolVariable, ByteVariable, Bytes32Variable, CircuitBuilder, CircuitVariable,
        PlonkParameters, U64Variable, Variable,
    },
};

use crate::utils::plonky2x_extensions::{bits_to_variable, exp_from_bits};

/// Returns the first 8 bytes of the hashed concatenation of seed with current_round
pub fn compute_pivot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    seed: Bytes32Variable,
    index_count: U64Variable,
    current_round: u8,
) -> U64Variable {
    let current_round_byte: ByteVariable = ByteVariable::constant(builder, current_round);
    let concatenation = [seed.as_bytes().as_slice(), &[current_round_byte]]
        .concat()
        .to_vec();

    let hash = builder.curta_sha256(&concatenation);

    let hash = U64Variable::decode(
        builder,
        &hash.as_bytes()[0..8]
            .into_iter()
            .rev()
            .cloned()
            .collect_vec(),
    );

    builder.rem(hash, index_count)
}

/// Returns the computation of (pivot + index_count - index) % index_count
pub fn compute_flip<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    pivot: U64Variable,
    index_count: U64Variable,
    index: U64Variable,
) -> U64Variable {
    let sum_pivot_index_count = builder.add(pivot, index_count);
    let sum_pivot_index_count_sub_index = builder.sub(sum_pivot_index_count, index);

    builder.rem(sum_pivot_index_count_sub_index, index_count)
}

/// Returns the hashed concatenation of seed, current_round and position divided by 256
pub fn compute_source<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    position: U64Variable,
    seed: Bytes32Variable,
    current_round: u8,
) -> Bytes32Variable {
    let current_round_byte = ByteVariable::constant(builder, current_round as u8);
    let const_256 = builder.constant::<U64Variable>(256);
    let position_div_256 = builder.div(position, const_256);
    let position_div_256_bytes = builder
        .to_le_bits(position_div_256)
        .chunks(8)
        .take(4)
        .map(|byte| ByteVariable(byte.iter().rev().cloned().collect_vec().try_into().unwrap()))
        .collect_vec();

    builder.curta_sha256(
        &[
            seed.as_bytes().as_slice(),
            &[current_round_byte],
            position_div_256_bytes.as_slice(),
        ]
        .concat(),
    )
}

/// Returns the byte in source at index (position % 256) / 8
pub fn compute_byte<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source_array: Bytes32Variable,
    position: U64Variable,
) -> ByteVariable {
    let const_8 = builder.constant::<U64Variable>(8);
    let const_256 = builder.constant::<U64Variable>(256);
    let position_mod_256 = builder.rem(position, const_256);
    let position_mod_256_div_8 = builder.div(position_mod_256, const_8);
    let position_mod_256_div_8_bits = builder.to_le_bits(position_mod_256_div_8);
    let position_mod_256_div_8_variable = bits_to_variable(builder, &position_mod_256_div_8_bits);

    builder.select_array(&source_array.0 .0, position_mod_256_div_8_variable)
}

/// Returns the remainder of byte / 2^(position % 8) and 2 as BoolVariable
pub fn compute_bit<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    byte: ByteVariable,
    position: U64Variable,
) -> BoolVariable {
    let const_0: Variable = builder.constant(L::Field::from_canonical_usize(0));
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let byte_to_variable = byte.to_variable(builder);
    let byte_u64 = U64Variable::from_variables(builder, &[byte_to_variable, const_0]);

    let position_first_3_bits = &builder.to_le_bits(position)[..3];
    let const_2_pow_position_first_3_bits = exp_from_bits(builder, const_2, &position_first_3_bits);
    let const_2_pow_position_first_3_bits_u64 =
        U64Variable::from_variables(builder, &[const_2_pow_position_first_3_bits, const_0]);
    let bit = builder.div(byte_u64, const_2_pow_position_first_3_bits_u64);

    builder.to_le_bits(bit)[0]
}
