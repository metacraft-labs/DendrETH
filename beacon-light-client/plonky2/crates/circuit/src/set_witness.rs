use plonky2::{
    hash::hash_types::{HashOut, HashOutTarget, RichField},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
};
use plonky2_crypto::biguint::{BigUintTarget, WitnessBigUint};

use crate::target_primitive::TargetPrimitive;

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

impl<F: RichField> SetWitness<F> for HashOutTarget {
    type Input = <Self as TargetPrimitive>::Primitive;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        let hashout = HashOut::from_vec(input.map(|number| F::from_canonical_u64(number)).to_vec());
        witness.set_hash_target(*self, hashout);
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

impl<F: RichField> SetWitness<F> for BigUintTarget {
    type Input = <Self as TargetPrimitive>::Primitive;

    fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
        witness.set_biguint_target(self, input);
    }
}
