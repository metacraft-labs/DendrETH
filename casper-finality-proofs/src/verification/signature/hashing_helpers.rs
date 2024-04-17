use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        ops::{BitAnd, BitXor},
        uint::num::u32::gadgets::arithmetic_u32::{CircuitBuilderU32, U32Target},
        vars::{BoolVariable, ByteVariable, BytesVariable, CircuitVariable, Variable},
    },
};

pub const SHA256_DIGEST_SIZE: u8 = 32;

// right shift := mul by power of 2, keep higher word
pub fn rsh_u32<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: U32Target,
    n: u8,
) -> U32Target {
    if n == 0 {
        return a;
    }
    let power_of_two = builder.api.constant_u32(0x1 << (32 - n));
    builder.api.mul_u32(a, power_of_two).1
}

pub fn and_u32<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    x: U32Target,
    y: U32Target,
) {
    let x_in_bits: Vec<BoolVariable> = builder
        .api
        .u32_to_bits_le(x)
        .into_iter()
        .map(BoolVariable::from)
        .collect();
    let y_in_bits: Vec<BoolVariable> = builder
        .api
        .u32_to_bits_le(y)
        .into_iter()
        .map(BoolVariable::from)
        .collect();

    let x_in_bits = x_in_bits
        .into_iter()
        .map(|x| x.variable)
        .collect::<Vec<Variable>>();

    let y_in_bits = y_in_bits
        .into_iter()
        .map(|y| y.variable)
        .collect::<Vec<Variable>>();

    let x_bytes: BytesVariable<4> = BytesVariable(
        x_in_bits
            .as_slice()
            .chunks_exact(8)
            .map(ByteVariable::from_variables_unsafe)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );
    let y_bytes: BytesVariable<4> = BytesVariable(
        y_in_bits
            .as_slice()
            .chunks_exact(8)
            .map(ByteVariable::from_variables_unsafe)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    let x = x_bytes.bitand(y_bytes, builder);
}

pub fn xor_u32<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    x: U32Target,
    y: U32Target,
) {
    let x_in_bits: Vec<BoolVariable> = builder
        .api
        .u32_to_bits_le(x)
        .into_iter()
        .map(BoolVariable::from)
        .collect();
    let y_in_bits: Vec<BoolVariable> = builder
        .api
        .u32_to_bits_le(y)
        .into_iter()
        .map(BoolVariable::from)
        .collect();

    let x_in_bits = x_in_bits
        .into_iter()
        .map(|x| x.variable)
        .collect::<Vec<Variable>>();

    let y_in_bits = y_in_bits
        .into_iter()
        .map(|y| y.variable)
        .collect::<Vec<Variable>>();

    let x_bytes: BytesVariable<4> = BytesVariable(
        x_in_bits
            .as_slice()
            .chunks_exact(8)
            .map(ByteVariable::from_variables_unsafe)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );
    let y_bytes: BytesVariable<4> = BytesVariable(
        y_in_bits
            .as_slice()
            .chunks_exact(8)
            .map(ByteVariable::from_variables_unsafe)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
    );

    let x = x_bytes.bitxor(y_bytes, builder);
}

pub fn add_virtual_hash_input_target() {}
pub fn hash_sha256() {}
