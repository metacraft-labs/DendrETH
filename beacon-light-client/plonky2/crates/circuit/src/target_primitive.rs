use plonky2::{
    hash::hash_types::{HashOutTarget, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

pub trait TargetPrimitive {
    type Primitive;
}

impl TargetPrimitive for Target {
    type Primitive = u64;
}

impl TargetPrimitive for BoolTarget {
    type Primitive = bool;
}

impl<T: TargetPrimitive, const N: usize> TargetPrimitive for [T; N] {
    type Primitive = [T::Primitive; N];
}

impl TargetPrimitive for HashOutTarget {
    type Primitive = [u64; NUM_HASH_OUT_ELTS];
}
