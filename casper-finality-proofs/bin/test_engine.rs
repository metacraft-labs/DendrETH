use std::collections::HashMap;

use casper_finality_proofs::test_engine::wrappers::{
    wrapper_hash_test::wrapper as wrapper_hash_test, wrapper_test::wrapper as wrapper_test,
};

struct TestCase {
    name: String,
    path: String,
}

impl TestCase {
    fn new(name: String, path: String) -> TestCase {
        TestCase { name, path }
    }
}

struct Mapper {
    folder_path: String,
    wrapper: Box<dyn Fn(String)>,
}

impl Mapper {
    fn new(folder_path: String, wrapper: Box<dyn Fn(String)>) -> Mapper {
        Mapper {
            folder_path,
            wrapper,
        }
    }
}

fn map_test_to_wrapper(test: &str) -> Box<dyn Fn(String)> {
    match test {
        "wrapper_test" => Box::new(|path| wrapper_test(path.as_str())),
        "wrapper_hash_test" => Box::new(|path| wrapper_hash_test(path.as_str())),
        _ => panic!("No wrapper found for test: {}", test),
    }
}

fn build_function_map(tests: Vec<TestCase>) -> HashMap<String, Box<Mapper>> {
    let mut function_map: HashMap<String, Box<Mapper>> = HashMap::new();

    for test in tests {
        let name = test.name.clone();
        function_map.insert(
            test.name,
            Box::new(Mapper::new(test.path, map_test_to_wrapper(name.as_str()))),
        );
    }

    function_map
}

fn main() {
    let mut tests: Vec<TestCase> = Vec::new();
    tests.push(TestCase::new(
        "wrapper_test".to_string(),
        "./src/test_engine/tests/test/".to_string(),
    ));
    tests.push(TestCase::new(
        "wrapper_hash_test".to_string(),
        "./src/test_engine/tests/hash_test/".to_string(),
    ));

    let function_map = build_function_map(tests);

    for (name, _) in &function_map {
        println!("Running circuit: {}", name);
        let folder_path = &function_map.get(name).unwrap().folder_path;
        let files = std::fs::read_dir(folder_path).unwrap();

        for file in files {
            let file = file.unwrap();
            let path = file.path().to_str().unwrap().to_owned();
            let file_name = file.file_name().to_str().unwrap().to_owned();
            println!("-> {}", file_name);
            (function_map.get(name).unwrap().wrapper)(path);
        }
    }
}
