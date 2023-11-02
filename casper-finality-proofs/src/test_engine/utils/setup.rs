use super::test_engine::TestCase;
use crate::test_engine::wrappers::{
    compute_shuffled_index::wrapper_mainnet::wrapper as wrapper_mainnet,
    compute_shuffled_index::wrapper_minimal::wrapper as wrapper_minimal,
    wrapper_hash_test::wrapper as wrapper_hash_test, wrapper_test::wrapper as wrapper_test,
    wrapper_test_lte::wrapper as wrapper_test_lte,
};
use strum::{Display, EnumString};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, EnumString, Display)]
pub enum TestWrappers {
    WrapperTest,
    WrapperHashTest,
    WrapperTestLte,
    WrapperMainnet,
    WrapperMinimal,
}

pub fn map_test_to_wrapper(
    test: TestWrappers,
) -> Box<dyn Fn(String, bool) -> Result<String, anyhow::Error> + Send + Sync> {
    match test {
        TestWrappers::WrapperTest => {
            Box::new(|data, should_assert| wrapper_test(data.as_str(), should_assert))
        }
        TestWrappers::WrapperHashTest => {
            Box::new(|path, should_assert| wrapper_hash_test(path.as_str(), should_assert))
        }
        TestWrappers::WrapperTestLte => {
            Box::new(|path, should_assert| wrapper_test_lte(path.as_str(), should_assert))
        }
        TestWrappers::WrapperMainnet => {
            Box::new(|path, should_assert| wrapper_mainnet(path.as_str(), should_assert))
        }
        TestWrappers::WrapperMinimal => {
            Box::new(|path, should_assert| wrapper_minimal(path.as_str(), should_assert))
        }
    }
}

pub fn init_tests() -> Vec<TestCase> {
    let mut tests: Vec<TestCase> = Vec::new();
    tests.push(TestCase::new(
        TestWrappers::WrapperTest,
        "./src/test_engine/tests/test/".to_string(),
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperHashTest,
        "./src/test_engine/tests/hash_test/".to_string(),
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperTestLte,
        "./src/test_engine/tests/test_lte/".to_string(),
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperMainnet,
        "./src/test_engine/tests/compute_shuffled_index/".to_string(),
    ));
    tests.push(TestCase::new(
        TestWrappers::WrapperMinimal,
        "./src/test_engine/tests/compute_shuffled_index/".to_string(),
    ));

    tests
}
