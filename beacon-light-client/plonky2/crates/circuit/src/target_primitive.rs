use plonky2::{
    hash::hash_types::{HashOutTarget, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::array::Array;

pub trait TargetPrimitive {
    type Primitive;
}

impl TargetPrimitive for Target {
    type Primitive = u64;
}

impl TargetPrimitive for BoolTarget {
    type Primitive = bool;
}

impl<T: TargetPrimitive + Serialize + DeserializeOwned, const N: usize> TargetPrimitive
    for Array<T, N>
where
    T::Primitive: Serialize + DeserializeOwned,
{
    type Primitive = Array<T::Primitive, N>;
}

impl TargetPrimitive for HashOutTarget {
    type Primitive = Array<u64, NUM_HASH_OUT_ELTS>;
}
