use plonky2::{
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
};

use crate::array::Array;

// TODO: new trait TargetPrimitiveType

pub trait SetWitness<F: RichField> {
    type Input;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input);
}

impl<F: RichField> SetWitness<F> for Target {
    type Input = u64;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        witness.set_target(*self, F::from_canonical_u64(*input));
    }
}

impl<F: RichField> SetWitness<F> for BoolTarget {
    type Input = bool;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        witness.set_bool_target(*self, *input);
    }
}

impl<F: RichField, T: SetWitness<F>, const N: usize> SetWitness<F> for [T; N] {
    type Input = Array<T::Input, N>;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        for (target, pw_input) in self.iter().zip(input.iter()) {
            target.set_witness(witness, pw_input);
        }
    }
}
