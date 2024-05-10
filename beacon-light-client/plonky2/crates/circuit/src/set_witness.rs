use plonky2::{
    hash::hash_types::RichField,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
};

use crate::target_primitive::TargetPrimitive;

// TODO: new trait TargetPrimitiveType

pub trait SetWitness<F: RichField> {
    type Input;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input);
}

impl<F: RichField> SetWitness<F> for Target {
    type Input = <Self as TargetPrimitive>::Primitive;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        witness.set_target(*self, F::from_canonical_u64(*input));
    }
}

impl<F: RichField> SetWitness<F> for BoolTarget {
    type Input = <Self as TargetPrimitive>::Primitive;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        witness.set_bool_target(*self, *input);
    }
}

impl<
        F: RichField,
        T: SetWitness<F> + TargetPrimitive<Primitive = <T as SetWitness<F>>::Input>,
        const N: usize,
    > SetWitness<F> for [T; N]
{
    type Input = <Self as TargetPrimitive>::Primitive;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        for (target, pw_input) in self.iter().zip(input.iter()) {
            target.set_witness(witness, pw_input);
        }
    }
}
