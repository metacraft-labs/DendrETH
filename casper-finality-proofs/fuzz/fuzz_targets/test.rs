#![no_main]

use casper_finality_proofs::test_engine::types::test_hash_data::Inputs;
use casper_finality_proofs::test_engine::wrappers::wrapper_hash_test::run;
use libfuzzer_sys::fuzz_target;

fn test(data: Inputs) {
    println!("data: {:?}", data);

    if !data.pubkey.is_zero() {
        let (hash, epoch) = run(data);

        println!("hash: {:?}, epoch: {}", hash, epoch);
    } else {
        panic!("pubkey is zero");
    }
}

fuzz_target!(|data: &[u8]| {
    bincode::deserialize::<Inputs>(data).ok().map(test);
});
