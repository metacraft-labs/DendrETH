use plonky2x::prelude::{BoolVariable, CircuitBuilder, PlonkParameters};

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
