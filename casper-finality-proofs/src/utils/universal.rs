use plonky2::iop::target::BoolTarget;
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, Variable, CircuitVariable};
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

pub fn div_rem<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    lhs: Variable,
    rhs: Variable,
) -> Variable {
    let quotient = builder.div(lhs, rhs);
    let quotient_times_rhs = builder.mul(quotient, rhs);

    builder.sub(rhs, quotient_times_rhs)
}

pub fn exp_from_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    base: Variable,
    exponent_bits: &[BoolVariable],
) -> Variable {
    Variable(builder.api.exp_from_bits(base.0, exponent_bits.into_iter()
    .map(|x| BoolTarget::new_unsafe(x.0 .0))))
}
