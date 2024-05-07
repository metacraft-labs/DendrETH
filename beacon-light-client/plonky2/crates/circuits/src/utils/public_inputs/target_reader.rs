use itertools::Itertools;
use plonky2::{
    hash::hash_types::{HashOutTarget, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

use crate::utils::{biguint::BigUintTarget, utils::biguint_target_from_limbs};

pub struct PublicInputsTargetReader<'a> {
    offset: usize,
    public_inputs: &'a [Target],
}

impl<'a> PublicInputsTargetReader<'a> {
    pub fn new(public_inputs: &'a [Target]) -> Self {
        Self {
            offset: 0,
            public_inputs,
        }
    }

    pub fn read(&mut self) -> Target {
        let target = self.public_inputs[self.offset];
        self.offset += 1;
        target
    }

    pub fn read_n(&mut self, n: usize) -> &'a [Target] {
        let read_targets = &self.public_inputs[self.offset..self.offset + n];
        self.offset += n;
        read_targets
    }

    pub fn read_object<R: PublicInputsTargetReadable>(&mut self) -> R {
        let read_targets = &self.public_inputs[self.offset..self.offset + R::get_size()];
        R::from_targets(&read_targets)
    }
}

pub trait PublicInputsTargetReadable {
    fn get_size() -> usize;
    fn from_targets(targets: &[Target]) -> Self;
}

impl PublicInputsTargetReadable for Target {
    fn get_size() -> usize {
        1
    }

    fn from_targets(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        targets[0]
    }
}

impl PublicInputsTargetReadable for BoolTarget {
    fn get_size() -> usize {
        1
    }

    fn from_targets(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        BoolTarget::new_unsafe(targets[0])
    }
}

impl PublicInputsTargetReadable for BigUintTarget {
    // TODO: make a Uint64 biguint wrapper
    fn get_size() -> usize {
        2
    }

    fn from_targets(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        biguint_target_from_limbs(targets)
    }
}

impl<R: PublicInputsTargetReadable + std::fmt::Debug, const N: usize> PublicInputsTargetReadable
    for [R; N]
{
    fn get_size() -> usize {
        R::get_size() * N
    }

    fn from_targets(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        let size = R::get_size();
        [(); N]
            .iter()
            .enumerate()
            .map(|(i, _)| R::from_targets(&targets[i * size..(i + 1) * size]))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

impl PublicInputsTargetReadable for HashOutTarget {
    fn get_size() -> usize {
        NUM_HASH_OUT_ELTS
    }

    fn from_targets(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        HashOutTarget::from_vec(targets.to_owned())
    }
}

trait PrimitivePublicInputsType {
    type PrimitiveType;
}
