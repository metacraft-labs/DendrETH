use plonky2x::{
    backend::circuit::Circuit,
    prelude::{
        CircuitBuilder, PlonkParameters, Variable
    },
};

#[derive(Debug, Clone)]
pub struct TestCircuit;

impl Circuit for TestCircuit {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let a = builder.read::<Variable>();
        let b = builder.read::<Variable>();

        let c = builder.add(a, b);

        builder.write::<Variable>(c);
    }
}

