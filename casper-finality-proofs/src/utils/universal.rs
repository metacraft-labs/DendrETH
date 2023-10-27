use plonky2::iop::target::BoolTarget;
use plonky2::field::types::Field;
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, Variable, CircuitVariable, Bytes32Variable};
use itertools::Itertools;

/// Fails if i1 != true.
pub fn assert_is_true<V: CircuitVariable, L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    i1: V
) {
    let one = builder.api.one();
    for t1 in i1.targets().iter() {
        builder.api.connect(*t1, one);
    }
}

/// Takes a slice of bits and returns the number with little-endian bit representation as a Variable.
pub fn le_sum<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable]
) -> Variable {
    let bits = bits
        .iter()
        .map(|x| BoolTarget::new_unsafe(x.0 .0))
        .collect_vec();
    Variable(builder.api.le_sum(bits.into_iter()))
}

/// Exponentiate `base` to the power of `exponent`, given by its little-endian bits.
pub fn exp_from_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    base: Variable,
    exponent_bits: &[BoolVariable],
) -> Variable {
    Variable(builder.api.exp_from_bits(base.0, exponent_bits.into_iter()
    .map(|x| BoolTarget::new_unsafe(x.0 .0))))
}

/// Converts Bytes32Variable's bits to little-endian bit representation and returns the accumulation of each bit by power of 2.
pub fn bytes32_to_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bytes: Bytes32Variable,
    start_idx: usize,
    end_idx: usize,
) -> Variable {
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
