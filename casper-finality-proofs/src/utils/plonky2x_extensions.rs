use plonky2::field::types::Field;
use plonky2::iop::target::BoolTarget;
use plonky2x::prelude::{
    BoolVariable, CircuitBuilder, CircuitVariable, PlonkParameters, U64Variable, Variable,
};
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

pub fn assert_is_not_equal<L: PlonkParameters<D>, const D: usize, V: CircuitVariable>(
    builder: &mut CircuitBuilder<L, D>,
    i1: V,
    i2: V,
) {
    let mut result = builder._false();
    for (t1, t2) in i1.targets().iter().zip(i2.targets().iter()) {
        let target_eq: BoolVariable = builder.api.is_equal(*t1, *t2).into();
        let target_not_eq = builder.not(target_eq);
        result = builder.or(target_not_eq, result);
    }

    assert_is_true(builder, result);
}

pub fn are_not_equal<L: PlonkParameters<D>, const D: usize, V: CircuitVariable>(
    builder: &mut CircuitBuilder<L, D>,
    i1: V,
    i2: V,
) -> BoolVariable {
    let are_same_pred = builder.is_equal(i1, i2);
    builder.not(are_same_pred)
}

/*
fn assert_distance_leq_n<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    greater: U64Variable,
    lesser: U64Variable,
    n: usize,
) {
    let diff = builder.sub(greater, lesser);
    let diff_one_pred = builder.is_equal(diff, one);
    let diff_two_pred = builder.is_equal(diff, two);
    let diff_one_or_two_pred = builder.or(diff_one_pred, diff_two_pred);
    assert_is_true(builder, diff_one_or_two_pred);
}
*/

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
