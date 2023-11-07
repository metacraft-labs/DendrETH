#![no_main]

use arbitrary::Result;
use casper_finality_proofs::test_engine::types::test_hash_data::Inputs;
use casper_finality_proofs::test_engine::wrappers::wrapper_hash_test::run;
use ethers::types::H256;
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use primitive_types::H384;

#[derive(Debug)]

struct ArbitraryH384(pub H384);

impl Arbitrary<'_> for ArbitraryH384 {
    fn arbitrary(u: &mut Unstructured) -> Result<Self> {
        let mut buf = [0u8; 48];
        u.fill_buffer(&mut buf)?;
        let h384 = H384::from_slice(&buf);
        Ok(ArbitraryH384(h384))
    }
}

#[derive(Debug)]
struct ArbitraryH256(pub H256);

impl Arbitrary<'_> for ArbitraryH256 {
    fn arbitrary(u: &mut Unstructured) -> Result<Self> {
        let mut buf = [0u8; 32];
        u.fill_buffer(&mut buf)?;
        let h256 = H256::from_slice(&buf);
        Ok(ArbitraryH256(h256))
    }
}

// #[derive(Arbitrary)]
#[derive(Debug)]
pub struct InputTest(pub Inputs);

impl Arbitrary<'_> for InputTest {
    fn arbitrary(u: &mut Unstructured) -> Result<Self> {
        // let mut byte_data = [0u8; 145];
        // u.fill_buffer(&mut byte_data)?;

        // let pubkey = H384::from_slice(&byte_data[0..48]);

        // let slashed = byte_data[48] % 2 == 0;

        // let a = H256::from_slice(&byte_data[49..81]);

        // let b = H256::from_slice(&byte_data[81..113]);

        // let mut slot = 0u64;
        // for &byte in &byte_data[113..145] {
        //     slot = (slot << 8) | byte as u64;
        // }

        let pubkey = ArbitraryH384::arbitrary(u)?.0;
        let slashed = bool::arbitrary(u)?;
        let a = ArbitraryH256::arbitrary(u)?.0;
        let b = ArbitraryH256::arbitrary(u)?.0;
        let slot = u64::arbitrary(u)?;

        Ok(InputTest(Inputs {
            pubkey,
            slashed,
            a,
            b,
            slot,
        }))
    }
}

fuzz_target!(|data: InputTest| {
    let (hash, epoch) = run(data.0);

    println!("hash: {:?}, epoch: {:?}", hash, epoch);
});
