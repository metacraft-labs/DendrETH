use plonky2::field::types::Field;
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, Variable};

pub fn bits_to_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: Vec<BoolVariable>,
    bits_len: usize
) -> Variable {
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let mut power_of_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut result = builder.constant(L::Field::from_canonical_usize(0));
    for i in 0..bits_len {
        let addend = builder.mul(bits[i].variable, power_of_2);
        result = builder.add(addend, result);
        power_of_2 = builder.mul(const_2, power_of_2);
    }

    result
}
