#![no_main]

mod utils {
    pub mod arbitrary_types;
}

use casper_finality_proofs::test_engine::wrappers::compute_shuffled_index::wrapper_mainnet::{
    run, MAINNET_CIRCUIT,
};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;
use std::env::var;
use utils::arbitrary_types::ArbitraryH256;

#[derive(Debug, arbitrary::Arbitrary)]
struct TestData {
    pub seed: ArbitraryH256,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=var("MAX_COUNT_COMPUTE_SHUFFLED_INDEX_MAINNET").unwrap().parse::<u8>().unwrap_or(15)))]
    pub count: u8,
}

fuzz_target!(|data: TestData| {
    Lazy::force(&MAINNET_CIRCUIT);
    println!("input: {:?}", data);
    let mut indices = Vec::<u64>::new();

    let count = data.count as u64;
    for i in 0..count {
        indices.push(run(i, count, data.seed.0));
    }

    assert!(indices.len() == count as usize);
    assert!(indices.iter().all(|&i| i < count));
    assert!(indices
        .iter()
        .all(|&i| indices.iter().filter(|&&j| j == i).count() == 1));
});
