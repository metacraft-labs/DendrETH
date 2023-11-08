use arbitrary::{Arbitrary, Result, Unstructured};
use primitive_types::{H256, H384};

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
