use crate::assert_equal;
use crate::compute_shuffled_index::compute_shuffled_index::ComputeShuffledIndex;
use crate::test_engine::types::compute_shuffled_index_data::TestData;
use crate::test_engine::utils::parse_file::read_fixture;
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
    ComputeShuffledIndex::define(&mut builder);
    builder.build()
});

pub fn wrapper(path: &str, should_assert: bool) -> Result<String, anyhow::Error> {
    type L = DefaultParameters;
    const D: usize = 2;

    let json_data: TestData = read_fixture::<TestData>(path);

    let mut input = CIRCUIT.input();
    let mut result_indices: Vec<u64> = Vec::new();

    for i in START_IDX..COUNT {
        let mut input = CIRCUIT.input();

        input.write::<U64Variable>(i);
        input.write::<U64Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(
            json_data.inputs.count,
        ));
        input.write::<Bytes32Variable>(json_data.inputs.seed);

        let (_witness, mut _output) = CIRCUIT.mock_prove(&input);
        let shuffled_index_res = _output.read::<U64Variable>();

        if should_assert {
            assert_equal!(json_data.outputs.mapping[i as usize], shuffled_index_res);
        }

        result_indices.push(shuffled_index_res);
    }

    Ok(format!("{:?}", result_indices))
}
