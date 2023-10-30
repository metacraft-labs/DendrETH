use plonky2x::prelude::{U64Variable, Bytes32Variable, PlonkParameters, CircuitBuilder, ByteVariable, BytesVariable, CircuitVariable, Variable, BoolVariable};
use plonky2::field::types::Field;
use itertools::Itertools;
use crate::utils::{utils::exp_from_bits, variable::bits_to_variable};

pub fn compute_hash<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    seed: Bytes32Variable,
    current_round: usize
) -> Bytes32Variable {
    let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
    let current_round_bytes: ByteVariable = ByteVariable::constant(builder, current_round as u8);

    let mut hash: BytesVariable<33> = BytesVariable([const_0_byte; 33]);
    for i in 0..32 {
        hash.0[i] = seed.0 .0[i];
    }
    hash.0[32] = current_round_bytes;

    builder.sha256(&hash.0)
}

pub fn compute_pivot<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    hash: Bytes32Variable,
    index_count: U64Variable
) -> U64Variable {
    let first_half_hash_bits = bytes_slice_to_variable(builder, hash, 0, 4);
    let second_half_hash_bits = bytes_slice_to_variable(builder, hash, 4, 8);

    let first_8_bytes_hash = U64Variable::from_variables(
        builder,
        &[
            second_half_hash_bits,
            first_half_hash_bits,
        ],
    );

    builder.rem(
        first_8_bytes_hash,
        index_count,
    )
}

pub fn compute_source<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    position: U64Variable,
    seed: Bytes32Variable,
    current_round: usize,
) -> Bytes32Variable {
    let current_round_bytes: ByteVariable = ByteVariable::constant(builder, current_round as u8);
    let const_256_u64 = builder.constant::<U64Variable>(256);
    let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
    let position_div_256 = builder.div(position, const_256_u64);
    let position_div_256_bits = builder.to_le_bits(position_div_256);
    let mut position_div_256_bytes = Vec::new();

    for i in 0..4 {
        let bits = position_div_256_bits[(i * 8)..((i + 1) * 8)]
            .iter()
            .rev()
            .map(|x| x.0)
            .collect_vec();
        position_div_256_bytes.push(ByteVariable::from_variables(builder, bits.as_slice()));
    }

    let mut source: BytesVariable<37> = BytesVariable([const_0_byte; 37]);
    for i in 0..32 {
        source.0[i] = seed.0 .0[i];
    }
    source.0[32] = current_round_bytes;
    for i in 0..4 {
        source.0[33 + i] = position_div_256_bytes[i];
    }

    builder.sha256(&source.0)
}

pub fn compute_byte<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    source_array: Bytes32Variable,
    position: U64Variable
) -> ByteVariable {
    let const_8_u64 = builder.constant::<U64Variable>(8);
    let const_256_u64 = builder.constant::<U64Variable>(256);
    let position_mod_256 = builder.rem(position, const_256_u64);
    let position_mod_256_div_8 = builder.div(position_mod_256, const_8_u64);
    let position_mod_256_div_8_bits = builder.to_le_bits(position_mod_256_div_8);
    let position_mod_256_div_8_variable = bits_to_variable(builder, position_mod_256_div_8_bits, 64);

    builder.select_array(&source_array.0 .0, position_mod_256_div_8_variable)
}

pub fn compute_bit<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    byte: ByteVariable,
    position: U64Variable
) -> U64Variable {
    let const_0: Variable = builder.constant(L::Field::from_canonical_usize(0));
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let const_2_u64 = builder.constant::<U64Variable>(2);
    let const_8_u64 = builder.constant::<U64Variable>(8);
    let byte_to_variable = byte.to_variable(builder);
    let byte_u64 = U64Variable::from_variables(builder, &[byte_to_variable, const_0]);

    let position_mod_8 = builder.rem(position, const_8_u64);
    let position_mod_8_to_bits = builder.to_le_bits(position_mod_8);
    let const_2_pow_position_mod_8 =
        exp_from_bits(builder, const_2, &position_mod_8_to_bits);
    let const_2_pow_position_mod_8_u64 =
        U64Variable::from_variables(builder, &[const_2_pow_position_mod_8, const_0]);
    let byte_shr_position_mod_8 = builder.div(byte_u64, const_2_pow_position_mod_8_u64);


    builder.rem(byte_shr_position_mod_8, const_2_u64)
}

/// Converts first 8 bytes of Bytes32Variable's bits to little-endian bit representation and returns the accumulation of each bit by power of 2.
pub fn bytes_slice_to_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bytes: Bytes32Variable,
    start_idx: usize,
    end_idx: usize,
) -> Variable {
    assert!(start_idx < end_idx);
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let mut power_of_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut result = builder.constant(L::Field::from_canonical_usize(0));
    let mut bits: Vec<BoolVariable> = Vec::new();

    for i in start_idx..end_idx {
        for j in 0..8 {
            bits.push(bytes.0 .0[7 - i].0[j]);
        }
    }

    for i in 0..32 {
        let addend = builder.mul(bits[31 - i].0, power_of_2);
        result = builder.add(addend, result);
        power_of_2 = builder.mul(const_2, power_of_2);
    }

    result
}
