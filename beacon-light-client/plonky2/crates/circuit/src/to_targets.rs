use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::array::Array;

pub trait ToTargets {
    fn to_targets(&self) -> Vec<Target>;
}

impl ToTargets for Target {
    fn to_targets(&self) -> Vec<Target> {
        vec![*self]
    }
}

impl ToTargets for BoolTarget {
    fn to_targets(&self) -> Vec<Target> {
        vec![self.target]
    }
}

impl<T: ToTargets + Serialize + DeserializeOwned, const N: usize> ToTargets for Array<T, N> {
    fn to_targets(&self) -> Vec<Target> {
        self.iter().fold(vec![], |mut acc, elem| {
            acc.extend(elem.to_targets());
            acc
        })
    }
}

impl ToTargets for HashOutTarget {
    fn to_targets(&self) -> Vec<Target> {
        self.elements.to_vec()
    }
}
