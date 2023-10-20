use crate::test_engine::types::test_lte_data::TestInput;
use crate::test_engine::utils::parse_file::read_fixture;
use crate::test_lte::TestCircuit;
use once_cell::sync::Lazy;
use plonky2x::backend::circuit::CircuitBuild;
use plonky2x::prelude::Field;
use plonky2x::{
    backend::circuit::Circuit,
    prelude::{CircuitBuilder, DefaultParameters, PlonkParameters, Variable},
};

// Singleton-like pattern
static CIRCUIT: Lazy<CircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    TestCircuit::define(&mut builder);
    builder.build()
});

pub fn wrapper(path: &str, _: bool) -> Result<String, anyhow::Error> {
    type L = DefaultParameters;
    const D: usize = 2;
    let json_data: TestInput = read_fixture::<TestInput>(path);

    let mut input = CIRCUIT.input();
    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
        json_data.a,
    ));
    input.write::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
        json_data.b,
    ));

    let (proof, output) = CIRCUIT.prove(&input);
    CIRCUIT.verify(&proof, &input, &output);
    Ok("".to_string())
}
