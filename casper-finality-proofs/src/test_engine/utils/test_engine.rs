use std::collections::HashMap;

use crate::test_engine::wrappers::{
    wrapper_hash_test::wrapper as wrapper_hash_test, wrapper_test::wrapper as wrapper_test,
};

pub struct TestCase {
    name: TestWrappers,
    path: String,
}

impl TestCase {
    fn new(name: TestWrappers, path: String) -> TestCase {
        TestCase { name, path }
    }
}

pub struct Mapper {
    pub folder_path: String,
    pub wrapper: Box<dyn Fn(String) -> Result<(), anyhow::Error>>,
}

impl Mapper {
    fn new(
        folder_path: String,
        wrapper: Box<dyn Fn(String) -> Result<(), anyhow::Error>>,
    ) -> Mapper {
        Mapper {
            folder_path,
            wrapper,
        }
    }
}

pub fn build_function_map(tests: Vec<TestCase>) -> HashMap<TestWrappers, Box<Mapper>> {
    let mut function_map: HashMap<TestWrappers, Box<Mapper>> = HashMap::new();

    for test in tests {
        function_map.insert(
            test.name,
            Box::new(Mapper::new(test.path, map_test_to_wrapper(test.name))),
        );
    }

    function_map
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
pub enum TestWrappers {
    WrapperTest,
    WrapperHashTest,
}

pub fn map_test_to_wrapper(test: TestWrappers) -> Box<dyn Fn(String) -> Result<(), anyhow::Error>> {
    match test {
        TestWrappers::WrapperTest => Box::new(|path| wrapper_test(path.as_str())),
        TestWrappers::WrapperHashTest => Box::new(|path| wrapper_hash_test(path.as_str())),
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

    tests
}
