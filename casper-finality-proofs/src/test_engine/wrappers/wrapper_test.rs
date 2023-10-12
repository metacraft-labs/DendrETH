use crate::test::TestCircuit;
use crate::test_engine::types::test_data::TestInput;
use crate::test_engine::utils::parse_file::read_fixture;
use plonky2x::prelude::Field;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};

pub fn wrapper(path: &str) {
    type L = DefaultParameters;
    const D: usize = 2;
    let json_data: TestInput = read_fixture::<TestInput>(path);

    let mut builder = CircuitBuilder::<L, D>::new();
    TestCircuit::define(&mut builder);
    let circuit = builder.build();
    let mut input = circuit.input();
    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
        json_data.inputs.a.as_u64(),
    ));
    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
        json_data.inputs.b.as_u64(),
    ));

    let (proof, mut output) = circuit.prove(&input);
    circuit.verify(&proof, &input, &output);
    let sum = output.read::<Variable>();
    assert_eq!(
        sum,
        <L as PlonkParameters<D>>::Field::from_canonical_u64(json_data.outputs.c.as_u64())
    );
}
