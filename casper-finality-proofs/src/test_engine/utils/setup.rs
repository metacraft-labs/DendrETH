use super::test_engine::TestCase;
use crate::test_engine::wrappers::{
    wrapper_hash_test::wrapper as wrapper_hash_test, wrapper_test::wrapper as wrapper_test,
    wrapper_test_lte::wrapper as wrapper_test_lte,
};

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub enum TestWrappers {
    WrapperTest,
    WrapperHashTest,
    WrapperTestLte,
}

pub fn map_test_to_wrapper(
    test: TestWrappers,
) -> Box<dyn Fn(String) -> Result<(), anyhow::Error> + Send + Sync> {
    match test {
        TestWrappers::WrapperTest => Box::new(|path| wrapper_test(path.as_str())),
        TestWrappers::WrapperHashTest => Box::new(|path| wrapper_hash_test(path.as_str())),
        TestWrappers::WrapperTestLte => Box::new(|path| wrapper_test_lte(path.as_str())),
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

    tests
}
