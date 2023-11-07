use crate::assert_equal;
use crate::compute_shuffled_index::circuit::define;
use crate::test_engine::types::compute_shuffled_index_data::ComputeShuffledIndexData;
use crate::test_engine::utils::parsers::parse_file::read_fixture;
use once_cell::sync::Lazy;
use plonky2x::backend::circuit::MockCircuitBuild;
use plonky2x::prelude::{Bytes32Variable, U64Variable};
use plonky2x::prelude::{CircuitBuilder, DefaultParameters};

// Singleton-like pattern
pub static MINIMAL_CIRCUIT: Lazy<MockCircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    define(&mut builder, 10);
    builder.mock_build()
});

pub fn wrapper(path: &str, should_assert: bool) -> Result<String, anyhow::Error> {
    let json_data = read_fixture::<ComputeShuffledIndexData>(path);

    let mut result_indices: Vec<u64> = Vec::new();

    for i in 0..json_data.count {
        let mut input = MINIMAL_CIRCUIT.input();

        input.write::<U64Variable>(i);
        input.write::<U64Variable>(json_data.count);
        input.write::<Bytes32Variable>(json_data.seed);

        let (_witness, mut _output) = MINIMAL_CIRCUIT.mock_prove(&input);
        let shuffled_index_res = _output.read::<U64Variable>();
        if should_assert {
            assert_equal!(json_data.mapping[i as usize], shuffled_index_res);
        }

        result_indices.push(shuffled_index_res);
    }

    Ok(format!("{:?}", result_indices))
}
