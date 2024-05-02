use itertools::Itertools;
use plonky2::{
    hash::hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

use crate::utils::utils::biguint_target_from_limbs;

use super::biguint::BigUintTarget;

pub struct PublicInputsReader<'a, F: RichField> {
    offset: usize,
    public_inputs: &'a [F],
}

impl<'a, F: RichField> PublicInputsReader<'a, F> {
    pub fn new(public_inputs: &'a [F]) -> Self {
        Self {
            offset: 0,
            public_inputs,
        }
    }

    pub fn read(&mut self) -> F {
        let element = self.public_inputs[self.offset];
        self.offset += 1;
        element
    }

    pub fn read_n(&mut self, n: usize) -> &'a [F] {
        let read_elements = &self.public_inputs[self.offset..self.offset + n];
        self.offset += n;
        read_elements
    }
}

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
        R::parse(&read_targets)
    }
}

pub trait PublicInputsTargetReadable {
    fn get_size() -> usize;
    fn parse(targets: &[Target]) -> Self;
}

impl PublicInputsTargetReadable for Target {
    fn get_size() -> usize {
        1
    }

    fn parse(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        targets[0]
    }
}

impl PublicInputsTargetReadable for BoolTarget {
    fn get_size() -> usize {
        1
    }

    fn parse(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        BoolTarget::new_unsafe(targets[0])
    }
}

impl PublicInputsTargetReadable for BigUintTarget {
    // TODO: make a Uint64 biguint wrapper
    fn get_size() -> usize {
        2
    }

    fn parse(targets: &[Target]) -> Self {
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

    fn parse(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        let size = Self::get_size();
        [(); N]
            .iter()
            .enumerate()
            .map(|(i, _)| R::parse(&targets[i * size..(i + 1) * size]))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

impl PublicInputsTargetReadable for HashOutTarget {
    fn get_size() -> usize {
        NUM_HASH_OUT_ELTS
    }

    fn parse(targets: &[Target]) -> Self {
        assert_eq!(targets.len(), Self::get_size());
        HashOutTarget::from_vec(targets.to_owned())
    }
}
