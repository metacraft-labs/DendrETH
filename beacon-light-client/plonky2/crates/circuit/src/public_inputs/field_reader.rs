use itertools::Itertools;
use plonky2::{
    hash::hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

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

    pub fn read_object<O: PublicInputsReadable>(&mut self) -> O::PrimitiveType {
        let read_elements = self.read_n(O::get_size());
        O::from_elements(read_elements)
    }
}

pub trait PublicInputsReadable: PublicInputsTargetReadable {
    type PrimitiveType;

    fn from_elements<F: RichField>(elements: &[F]) -> Self::PrimitiveType;
}

impl PublicInputsReadable for Target {
    type PrimitiveType = u64;

    fn from_elements<F: RichField>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64()
    }
}

impl PublicInputsReadable for BoolTarget {
    type PrimitiveType = bool;

    fn from_elements<F: RichField>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64() != 0
    }
}

impl<R: PublicInputsReadable + std::fmt::Debug, const N: usize> PublicInputsReadable for [R; N]
where
    <R as PublicInputsReadable>::PrimitiveType: std::fmt::Debug,
{
    type PrimitiveType = [R::PrimitiveType; N];

    fn from_elements<F: RichField>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        let size = R::get_size();
        [(); N]
            .iter()
            .enumerate()
            .map(|(i, _)| R::from_elements(&elements[i * size..(i + 1) * size]))
            .collect_vec()
            .try_into()
            .unwrap()
    }
}

impl PublicInputsReadable for HashOutTarget {
    type PrimitiveType = [u64; NUM_HASH_OUT_ELTS];

    fn from_elements<F: RichField>(elements: &[F]) -> Self::PrimitiveType {
        assert_eq!(elements.len(), Self::get_size());
        elements
            .into_iter()
            .map(|elem| elem.to_canonical_u64())
            .collect_vec()
            .try_into()
            .unwrap()
    }
}
