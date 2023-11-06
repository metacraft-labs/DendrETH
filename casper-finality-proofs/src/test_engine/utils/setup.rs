use super::test_engine::TestCase;
use crate::test_engine::wrappers::wrapper_weigh_justification_and_finalization::{
    wrapper as wrapper_weigh_justification_and_finalization,
    CIRCUIT as circuit_weigh_justification_and_finalization,
};
use once_cell::sync::Lazy;
use strum::{Display, EnumString};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, EnumString, Display)]
pub enum TestWrappers {
    WrapperWeighJustificationAndFinalizationConsensusMainnet,
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
    }
}

pub fn init_tests() -> Vec<TestCase> {
    let mut tests: Vec<TestCase> = Vec::new();
    tests.push(TestCase::new(
        TestWrappers::WrapperWeighJustificationAndFinalizationConsensusMainnet,
        "../vendor/consensus-spec-tests/tests/mainnet/capella/epoch_processing/justification_and_finalization/pyspec_tests/".to_string(),
        true,
    ));

    tests
}
