use crate::assert_equal;
use crate::compute_shuffled_index::compute_shuffled_index::ComputeShuffledIndex;
use crate::test_engine::types::compute_shuffled_index_data::TestData;
use crate::test_engine::utils::parse_file::read_fixture;
use once_cell::sync::Lazy;
use plonky2x::backend::circuit::MockCircuitBuild;
use plonky2x::prelude::{Bytes32Variable, U64Variable};
use plonky2x::prelude::{CircuitBuilder, DefaultParameters};

// Singleton-like pattern
static CIRCUIT: Lazy<MockCircuitBuild<DefaultParameters, 2>> = Lazy::new(|| {
    const SHUFFLE_ROUND_COUNT: usize = 90;
    let mut builder = CircuitBuilder::<DefaultParameters, 2>::new();
    ComputeShuffledIndex::define_circuit(&mut builder, SHUFFLE_ROUND_COUNT);
    builder.mock_build()
});

pub fn wrapper(path: &str, should_assert: bool) -> Result<String, anyhow::Error> {
    let json_data: TestData = read_fixture::<TestData>(path);

    let mut result_indices: Vec<u64> = Vec::new();

    for i in 0..json_data.count {
        let mut input = CIRCUIT.input();

        input.write::<U64Variable>(i);
        input.write::<U64Variable>(json_data.count);
        input.write::<Bytes32Variable>(json_data.seed);

        let (_witness, mut _output) = CIRCUIT.mock_prove(&input);
        let shuffled_index_res = _output.read::<U64Variable>();
        if should_assert {
            assert_equal!(json_data.mapping[i as usize], shuffled_index_res);
        }

        result_indices.push(shuffled_index_res);
    }

    Ok(format!("{:?}", result_indices))
}
