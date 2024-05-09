use plonky2::{
    hash::hash_types::HashOutTarget,
    iop::target::{BoolTarget, Target},
};

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

impl<T: ToTargets, const N: usize> ToTargets for [T; N] {
    fn to_targets(&self) -> Vec<Target> {
        self.into_iter().fold(vec![], |mut acc, elem| {
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
