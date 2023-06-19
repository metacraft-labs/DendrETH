use std::{marker::PhantomData, println};

use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    hash::{
        hash_types::{RichField},
        hashing::{PlonkyPermutation, SPONGE_WIDTH},
        merkle_tree::MerkleTree,
    },
    plonk::config::{GenericHashOut, Hasher},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdditionPermutation<F: RichField> {
    phantom: PhantomData<F>,
}

impl<F: RichField> PlonkyPermutation<F> for AdditionPermutation<F> {
    fn permute(input: [F; SPONGE_WIDTH]) -> [F; SPONGE_WIDTH] {
        let mut output = input;
        output.rotate_left(1);
        output
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(bound = "")]
pub struct AdditionHash<F: RichField>(F);

impl GenericHashOut<GoldilocksField> for AdditionHash<GoldilocksField> {
    fn to_bytes(&self) -> Vec<u8> {
        let bytes = self.0.0.to_le_bytes().to_vec();
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut array = [0u8; 8];
        let bytes = &bytes[..array.len()]; // panics if not enough input
        array.copy_from_slice(bytes);
        let num = u64::from_le_bytes(array);
        AdditionHash(GoldilocksField::from_canonical_u64(num))
    }

    fn to_vec(&self) -> Vec<GoldilocksField> {
        vec![self.0]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdditionHasher {
    phantom: PhantomData<GoldilocksField>,
}

impl Hasher<GoldilocksField> for AdditionHasher {
    const HASH_SIZE: usize = std::mem::size_of::<GoldilocksField>();

    type Hash = AdditionHash<GoldilocksField>;
    type Permutation = AdditionPermutation<GoldilocksField>;

    fn hash_no_pad(input: &[GoldilocksField]) -> Self::Hash {
        AdditionHash(input.iter().fold(GoldilocksField::ZERO, |acc, x| acc + *x))
    }

    fn hash_public_inputs(input: &[GoldilocksField]) -> Self::Hash {
        Self::hash_no_pad(input)
    }

    fn two_to_one(left: Self::Hash, right: Self::Hash) -> Self::Hash {
        AdditionHash(left.0 + right.0)
    }
}

fn main() {
    type F = GoldilocksField;

    let merkle_tree = MerkleTree::<F, AdditionHasher>::new(
        vec![
            vec![F::from_canonical_u32(1)],
            vec![F::from_canonical_u32(2)],
            vec![F::from_canonical_u32(3)],
            vec![F::from_canonical_u32(4)],
            vec![F::from_canonical_u32(5)],
            vec![F::from_canonical_u32(6)],
            vec![F::from_canonical_u32(7)],
            vec![F::from_canonical_u32(8)],
            vec![F::from_canonical_u32(9)],
            vec![F::from_canonical_u32(10)],
            vec![F::from_canonical_u32(11)],
            vec![F::from_canonical_u32(12)],
            vec![F::from_canonical_u32(13)],
            vec![F::from_canonical_u32(14)],
            vec![F::from_canonical_u32(15)],
            vec![F::from_canonical_u32(16)],
        ],
        0,
    );

    let proof = merkle_tree.prove(3);

    println!("{:?}", proof);

    println!("{:?}", merkle_tree.digests);
}
