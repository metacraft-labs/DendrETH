#![no_main]

mod utils {
    pub mod arbitrary_types;
}

use casper_finality_proofs::test_engine::wrappers::compute_shuffled_index::wrapper_mainnet::{
    run, MAINNET_CIRCUIT,
};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use lighthouse_swap_or_not_shuffle::compute_shuffled_index;
use once_cell::sync::Lazy;
use serde_derive::Serialize;
use std::env::var;
use utils::arbitrary_types::ArbitraryH256;

#[derive(Debug, arbitrary::Arbitrary, Serialize)]
struct TestData {
    pub seed: ArbitraryH256,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=var("MAX_COUNT_COMPUTE_SHUFFLED_INDEX_MAINNET").unwrap().parse::<u8>().unwrap_or(15)))]
    pub count: u8,
}

fuzz_target!(|data: TestData| {
    let time = std::time::Instant::now();
    Lazy::force(&MAINNET_CIRCUIT);

    let mut indices = Vec::<u64>::new();

    let count = data.count as u64;
    println!("\ncount: {}", count);
    for i in 0..count {
        let output = run(i, count, data.seed.0);
        let output_ref =
            compute_shuffled_index(i as usize, count as usize, data.seed.0.as_bytes(), 90).unwrap()
                as u64;
        assert_eq!(output, output_ref);

        indices.push(output);
    }

    assert!(indices.len() == count as usize);
    assert!(indices.iter().all(|&i| i < count));
    assert!(indices
        .iter()
        .all(|&i| indices.iter().filter(|&&j| j == i).count() == 1));

    println!("test took: {:?}", time.elapsed());
});
