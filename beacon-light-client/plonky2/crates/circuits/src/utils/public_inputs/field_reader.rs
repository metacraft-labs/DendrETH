use itertools::Itertools;
use num::BigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

use crate::utils::biguint::BigUintTarget;

use super::target_reader::PublicInputsTargetReadable;

pub struct PublicInputsFieldReader<'a, F: RichField> {
    offset: usize,
    public_inputs: &'a [F],
}

impl<'a, F: RichField> PublicInputsFieldReader<'a, F> {
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

pub trait PublicInputsReadable: PublicInputsTargetReadable {
    type PrimitiveType;

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType;
}

impl PublicInputsReadable for Target {
    type PrimitiveType = u64;

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64()
    }
}

impl PublicInputsReadable for BoolTarget {
    type PrimitiveType = bool;

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64() != 0
    }
}

impl PublicInputsReadable for BigUintTarget {
    // TODO: make a Uint64 biguint wrapper
    type PrimitiveType = u64;

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        let first_limb = elements[0].to_canonical_u64();
        let second_limb = elements[1].to_canonical_u64();
        first_limb + (second_limb << 32)
    }
}

impl<R: PublicInputsReadable + std::fmt::Debug, const N: usize> PublicInputsReadable for [R; N] {
    type PrimitiveType = [R::PrimitiveType; N];

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        let size = R::get_size();
        [(); N]
            .iter()
            .enumerate()
            .map(|(i, _)| R::parse(&elements[i * size..(i + 1) * size]))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

impl PublicInputsReadable for HashOutTarget {
    type PrimitiveType = [u64; NUM_HASH_OUT_ELTS];

    fn parse<F: RichField + Extendable<D>, const D: usize>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements
            .into_iter()
            .map(|elem| elem.to_canonical_u64())
            .collect()
    }
}
