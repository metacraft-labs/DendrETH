use super::test_engine::TestCase;
use crate::test_engine::wrappers::compute_shuffled_index::wrapper_mainnet::{
    wrapper as wrapper_mainnet, MAINNET_CIRCUIT as circuit_mainnet,
};
use crate::test_engine::wrappers::compute_shuffled_index::wrapper_minimal::{
    wrapper as wrapper_minimal, MINIMAL_CIRCUIT as circuit_minimal,
};
use crate::test_engine::wrappers::wrapper_prove_finality::{
    wrapper as wrapper_prove_finality, CIRCUIT as circuit_prove_finality,
};
use crate::test_engine::wrappers::wrapper_weigh_justification_and_finalization::{
    wrapper as wrapper_weigh_justification_and_finalization,
    CIRCUIT as circuit_weigh_justification_and_finalization,
};
use once_cell::sync::Lazy;
use strum::{Display, EnumString};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, EnumString, Display)]
pub enum TestWrappers {
    WrapperComputeShuffledIndexConsensusMainnet,
    WrapperComputeShuffledIndexConsensusMinimal,
    WrapperWeighJustificationAndFinalizationConsensusMainnet,
    WrapperProveFinality,
}

pub fn map_test_to_wrapper(
    test: TestWrappers,
) -> (
    Box<dyn Fn() -> () + Send + Sync>,
    Box<dyn Fn(String, bool) -> Result<String, anyhow::Error> + Send + Sync>,
) {
    match test {
        TestWrappers::WrapperWeighJustificationAndFinalizationConsensusMainnet => (
            Box::new(|| {
                Lazy::force(&circuit_weigh_justification_and_finalization);
            }),
            Box::new(|path, should_assert| {
                wrapper_weigh_justification_and_finalization(path, should_assert)
            }),
        ),
        TestWrappers::WrapperComputeShuffledIndexConsensusMainnet => (
            Box::new(|| {
                Lazy::force(&circuit_mainnet);
            }),
            Box::new(|path, should_assert| wrapper_mainnet(&path, should_assert)),
        ),
        TestWrappers::WrapperComputeShuffledIndexConsensusMinimal => (
            Box::new(|| {
                Lazy::force(&circuit_minimal);
            }),
            Box::new(|path, should_assert| wrapper_minimal(&path, should_assert)),
        ),
        TestWrappers::WrapperProveFinality => (
            Box::new(|| {
                Lazy::force(&circuit_prove_finality);
            }),
            Box::new(|path, should_assert| wrapper_prove_finality(&path, should_assert)),
        ),
    }
}

pub fn init_tests() -> Vec<TestCase> {
    let mut tests: Vec<TestCase> = Vec::new();
    tests.push(TestCase::new(
        TestWrappers::WrapperWeighJustificationAndFinalizationConsensusMainnet,
        "../vendor/consensus-spec-tests/tests/mainnet/capella/epoch_processing/justification_and_finalization/pyspec_tests/".to_string(),
        true,
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperComputeShuffledIndexConsensusMainnet,
        "../vendor/consensus-spec-tests/tests/mainnet/phase0/shuffling/core/shuffle".to_string(),
        false,
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperComputeShuffledIndexConsensusMinimal,
        "../vendor/consensus-spec-tests/tests/minimal/phase0/shuffling/core/shuffle".to_string(),
        false,
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperProveFinality,
        "./src/test_engine/tests/test".to_string(),
        false,
    ));

    tests
}
