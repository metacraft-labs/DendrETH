use plonky2::iop::target::BoolTarget;
use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters, Variable, CircuitVariable};

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

/// Exponentiate `base` to the power of `exponent`, given by its little-endian bits.
pub fn exp_from_bits<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    base: Variable,
    exponent_bits: &[BoolVariable],
) -> Variable {
    Variable(builder.api.exp_from_bits(base.0, exponent_bits.into_iter()
    .map(|x| BoolTarget::new_unsafe(x.variable.0))))
}

