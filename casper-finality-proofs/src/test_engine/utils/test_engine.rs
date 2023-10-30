use super::setup::{map_test_to_wrapper, TestWrappers};
use colored::{ColoredString, Colorize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct TestCase {
    pub name: TestWrappers,
    pub path: String,
}

impl TestCase {
    pub fn new(name: TestWrappers, path: String) -> TestCase {
        TestCase { name, path }
    }
}

pub struct Mapper {
    pub folder_path: String,
    pub wrapper: Box<dyn Fn(String, bool) -> Result<String, anyhow::Error> + Send + Sync>,
}

impl Mapper {
    fn new(
        folder_path: String,
        wrapper: Box<dyn Fn(String, bool) -> Result<String, anyhow::Error> + Send + Sync>,
    ) -> Mapper {
        Mapper {
            folder_path,
            wrapper,
        }
    }
}

pub fn handle_error(
    e: Box<dyn std::any::Any + Send>,
    verbose: bool,
    file_name: &str,
    circuit_name: &ColoredString,
    failed_tests: &mut Vec<String>,
) {
    let mut error_str = String::from("Circuit failure");
    if let Some(e) = e.downcast_ref::<&'static str>() {
        error_str = format!("Error: {}", e);
    } else if let Some(e) = e.downcast_ref::<String>() {
        error_str = format!("Error: {}", e);
    } else if let Some(e) = e.downcast_ref::<anyhow::Error>() {
        error_str = format!("Error: {}", e);
    }
    if verbose {
        return println!(
            "{} {} {}",
            "[FAIL]".red().bold(),
            file_name.to_string().yellow(),
            error_str
        );
    }
    failed_tests.push(format!(
        "[{}] {}: {}",
        circuit_name,
        String::from(file_name).yellow(),
        error_str
    ));
    println!("-> {}", String::from(file_name).on_red());
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
