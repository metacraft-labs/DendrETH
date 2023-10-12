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
