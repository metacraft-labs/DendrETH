use plonky2::{
    hash::hash_types::{HashOutTarget, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

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

impl<T: TargetPrimitive, const N: usize> TargetPrimitive for [T; N] {
    type Primitive = Array<T::Primitive, N>;
}

impl TargetPrimitive for HashOutTarget {
    type Primitive = Array<u64, NUM_HASH_OUT_ELTS>;
}

// TODO: figure this out
// impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> TargetPrimitive
//     for ProofWithPublicInputsTarget<D>
// {
//     type Primitive = ProofWithPublicInputs<F, C, D>;
// }
