#![no_main]

mod utils {
    pub mod arbitrary_types;
}

use casper_finality_proofs::test_engine::wrappers::compute_shuffled_index::wrapper_minimal::{
    run, MINIMAL_CIRCUIT,
};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;
use utils::arbitrary_types::ArbitraryH256;

#[derive(Debug, arbitrary::Arbitrary)]
struct TestData {
    pub seed: ArbitraryH256,
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(1..=15))]
    pub count: u8,
}

fuzz_target!(|data: TestData| {
    Lazy::force(&MINIMAL_CIRCUIT);

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
