use itertools::Itertools;
use plonky2::{
    hash::hash_types::{HashOutTarget, RichField, NUM_HASH_OUT_ELTS},
    iop::target::{BoolTarget, Target},
};

use crate::{array::Array, target_primitive::TargetPrimitive};

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

    pub fn read_object<R: PublicInputsReadable>(&mut self) -> R::Primitive {
        let read_elements = self.read_n(R::get_size());
        R::from_elements(read_elements)
    }
}

pub trait PublicInputsReadable: PublicInputsTargetReadable + TargetPrimitive {
    fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive;
}

impl PublicInputsReadable for Target {
    fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64()
    }
}

impl PublicInputsReadable for BoolTarget {
    fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
        assert_eq!(elements.len(), Self::get_size());
        elements[0].to_canonical_u64() != 0
    }
}

impl<R: PublicInputsReadable + std::fmt::Debug, const N: usize> PublicInputsReadable for [R; N]
where
    R::Primitive: std::fmt::Debug,
{
    fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
        assert_eq!(elements.len(), Self::get_size());
        let size = R::get_size();
        Array::<R::Primitive, N>(
            [(); N]
                .iter()
                .enumerate()
                .map(|(i, _)| R::from_elements(&elements[i * size..(i + 1) * size]))
                .collect_vec()
                .try_into()
                .unwrap(),
        )
    }
}

impl PublicInputsReadable for HashOutTarget {
    fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
        assert_eq!(elements.len(), Self::get_size());
        Array::<u64, NUM_HASH_OUT_ELTS>(
            elements
                .into_iter()
                .map(|elem| elem.to_canonical_u64())
                .collect_vec()
                .try_into()
                .unwrap(),
        )
    }
}
