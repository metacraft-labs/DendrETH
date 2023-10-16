use casper_finality_proofs::test_engine::utils::setup::{map_test_to_wrapper, TestWrappers};
use std::env;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let circuit_name = &args[0];
    let file_path = &args[1];

    let circuit = TestWrappers::from_str(&circuit_name).unwrap();
    let wrapper = map_test_to_wrapper(circuit);

    let res = wrapper(file_path.to_string());
    match res {
        Ok(_) => {}
        Err(e) => panic!("Test failed: {:?}", e),
    }
}
