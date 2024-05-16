use plonky2::field::types::Field;
use plonky2::iop::target::BoolTarget;
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, U64Variable, Variable};
use std::cmp::min;

pub fn assert_is_true<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    condition: BoolVariable,
) {
    let _true = builder._true();
    builder.assert_is_equal(condition, _true);
}

pub fn assert_is_false<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    condition: BoolVariable,
) {
    let _false = builder._false();
    builder.assert_is_equal(condition, _false);
}

/// Returns the little endian representation of bits
pub fn bits_to_variable<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable],
) -> Variable {
    let const_2: Variable = builder.constant(L::Field::from_canonical_usize(2));
    let mut power_of_2 = builder.constant(L::Field::from_canonical_usize(1));
    let mut result = builder.constant(L::Field::from_canonical_usize(0));
    for i in 0..bits.len() {
        let addend = builder.mul(bits[i].variable, power_of_2);
        result = builder.add(addend, result);
        power_of_2 = builder.mul(const_2, power_of_2);
    }

    result
}

/// Exponentiate `base` with `exponent`, given its bits in little-endian.
pub fn exp_from_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    base: Variable,
    exponent_bits: &[BoolVariable],
) -> Variable {
    Variable(
        builder.api.exp_from_bits(
            base.0,
            exponent_bits
                .into_iter()
                .map(|x| BoolTarget::new_unsafe(x.variable.0)),
        ),
    )
}

/// Returns the greater one of the two arguments
pub fn max<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    lhs: U64Variable,
    rhs: U64Variable,
) -> U64Variable {
    let lhs_lte_rhs = builder.lte(lhs, rhs);
    builder.select(lhs_lte_rhs, rhs, lhs)
}

pub fn shift_right<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bits: &[BoolVariable],
    shift_count: usize,
) -> Vec<BoolVariable> {
    let mut new_bits = bits.to_vec();
    for i in shift_count..bits.len() {
        new_bits[i] = bits[i - shift_count];
    }

    for i in 0..min(shift_count, bits.len()) {
        new_bits[i] = builder._false();
    }

    new_bits
}

/// Split the given integer into a list of wires, where each one represents a
/// bit of the integer, with little-endian ordering.
pub fn variable_to_le_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    variable: Variable,
    num_bits: usize,
) -> Vec<BoolVariable> {
    builder
        .api
        .split_le(variable.0, num_bits)
        .into_iter()
        .map(|v| BoolVariable::from(v))
        .collect()
}

pub fn assert_zero<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    variable: Variable,
) {
    builder.api.assert_zero(variable.0)
}
