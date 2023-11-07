#![no_main]

use arbitrary::Result;
use casper_finality_proofs::test_engine::types::test_hash_data::Inputs;
use casper_finality_proofs::test_engine::wrappers::wrapper_hash_test::run;
use ethers::types::H256;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use primitive_types::H384;

#[derive(Debug)]
pub struct ArbitraryH256(pub H256);

impl Arbitrary<'_> for ArbitraryH256 {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let unstruct = u.arbitrary::<[u8; 32]>()?;
        let res = H256::from_slice(&unstruct);
        Ok(ArbitraryH256(res))
    }
}

#[derive(Debug)]
pub struct ArbitraryH384(pub H384);

impl Arbitrary<'_> for ArbitraryH384 {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let unstruct = u.arbitrary::<[u8; 48]>()?;
        let res = H384::from_slice(&unstruct);
        Ok(ArbitraryH384(res))
    }
}

#[derive(Debug, Arbitrary)]
pub struct InputTest {
    pub pubkey: ArbitraryH384,
    pub slashed: bool,
    pub a: ArbitraryH256,
    pub b: ArbitraryH256,
    pub slot: u64,
}

fuzz_target!(|data: InputTest| {
    if data.pubkey.0.is_zero() {
        return;
    }

    let data = Inputs {
        pubkey: data.pubkey.0,
        slashed: data.slashed,
        a: data.a.0,
        b: data.b.0,
        slot: data.slot,
    };

    println!("data: {:?}", data);

    let (hash, epoch) = run(data);

    println!("hash: {:?}, epoch: {}", hash, epoch);
});
