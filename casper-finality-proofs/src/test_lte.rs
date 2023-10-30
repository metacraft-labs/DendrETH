use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, PlonkParameters, Variable},
};

#[derive(Debug, Clone)]
pub struct TestCircuit;

impl Circuit for TestCircuit {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let a = builder.read::<Variable>();
        let b = builder.read::<Variable>();

        let c = builder.lte(a, b);
        let _true = builder._true();
        builder.assert_is_equal(c, _true);
    }
}
