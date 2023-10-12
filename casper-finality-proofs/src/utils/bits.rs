use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters};

pub fn shift_right<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable],
    shift_count: usize,
) -> Vec<BoolVariable> {
    let mut new_bits = bits.to_vec();
    for i in shift_count..bits.len() {
        new_bits[i] = bits[i - shift_count];
    }

    for i in 0..shift_count {
        new_bits[i] = builder._false();
    }

    new_bits
}
