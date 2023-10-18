use plonky2x::prelude::{PlonkParameters, ByteVariable, BoolVariable, CircuitBuilder, CircuitVariable};
use array_macro::array;

pub fn constant<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    value: u8,
) -> ByteVariable {
    ByteVariable(array![i => BoolVariable::constant(builder, (value >> (7 - i)) & 1 == 1); 8])
}
