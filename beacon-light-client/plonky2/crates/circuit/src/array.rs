use deriving_via::DerivingVia;
use serde::Deserialize;
use serde::Serialize;
use serde_big_array::BigArray;

#[derive(DerivingVia, Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct Array<T, const N: usize>(#[serde(with = "BigArray")] pub [T; N]);
