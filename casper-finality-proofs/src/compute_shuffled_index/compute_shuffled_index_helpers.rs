use plonky2x::{prelude::{U64Variable, Bytes32Variable, PlonkParameters, CircuitBuilder, ByteVariable, BytesVariable, CircuitVariable, Variable, BoolVariable}, frontend::vars::EvmVariable};
use plonky2::field::types::Field;
use itertools::Itertools;
use crate::utils::{utils::exp_from_bits, variable::bits_to_variable};

/// Returns the first 8 bytes of the bytes concatenation of seed and current_round as U64Variable
pub fn compute_hash<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    seed: Bytes32Variable,
    current_round: usize
) -> U64Variable {
    let const_0_byte: ByteVariable = ByteVariable::constant(builder, 0);
    let current_round_bytes: ByteVariable = ByteVariable::constant(builder, current_round as u8);

    let mut hash: BytesVariable<33> = BytesVariable([const_0_byte; 33]);
    for i in 0..32 {
        hash.0[i] = seed.0 .0[i];
    }
    hash.0[32] = current_round_bytes;

    let hash = builder.curta_sha256(&hash.0);

    U64Variable::decode(builder, &hash.0.0[0..8].iter().rev().cloned().collect_vec())
}

/// Converts position to variable and returns the bytes concatenation of seed, current_round and position divided by 256
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
            .map(|x| x.variable)
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

    builder.curta_sha256(&source.0)
}

/// Returns the byte in source at index (position % 256) / 8
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

/// Returns the remainder of byte / 2^(position % 8) and 2 as BoolVariable
pub fn compute_bit<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    byte: ByteVariable,
    position: U64Variable
) -> BoolVariable {
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
    let bit = builder.rem(byte_shr_position_mod_8, const_2_u64);

    BoolVariable::from_variables(builder, &[bit.variables()[0]])
}
