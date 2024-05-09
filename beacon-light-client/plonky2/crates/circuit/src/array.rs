use deriving_via::DerivingVia;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_big_array::BigArray;

#[derive(DerivingVia, Serialize, Deserialize)]
pub struct Array<T: DeserializeOwned + Serialize, const N: usize>(
    #[serde(with = "BigArray")] pub [T; N],
);
