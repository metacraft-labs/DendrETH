use plonky2::field::types::Field;
use plonky2x::prelude::{BoolVariable, ByteVariable, CircuitBuilder, PlonkParameters, Variable};
use itertools::Itertools;

pub fn to_byte_variable<L: PlonkParameters<D>, const D: usize>(
    variable: Variable,
    builder: &mut CircuitBuilder<L, D>,
) -> ByteVariable {
    let bits: [BoolVariable; 8] = to_bits(variable, builder);
    ByteVariable(bits)
}

pub fn to_bits<const N: usize, L: PlonkParameters<D>, const D: usize>(
    variable: Variable,
    builder: &mut CircuitBuilder<L, D>,
) -> [BoolVariable; N] {
    builder
        .api
        .split_le(variable.0, N)
        .iter()
        .rev()
        .map(|x| BoolVariable::from(x.target))
        .collect_vec()
        .try_into()
        .unwrap()
}

pub fn bits_to_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: Vec<BoolVariable>,
    bits_len: usize
) -> Variable {
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let mut power_of_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut result = builder.constant(L::Field::from_canonical_usize(0));
    for i in 0..bits_len {
        let addend = builder.mul(bits[i].0, power_of_2);
        result = builder.add(addend, result);
        power_of_2 = builder.mul(const_2, power_of_2);
    }

    result
}
