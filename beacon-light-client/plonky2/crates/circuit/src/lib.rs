#![feature(associated_type_defaults)]

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{target::Target, witness::PartialWitness},
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitData, config::GenericConfig},
    util::serialization::{Buffer, IoResult},
};

// TODO: stick this in CircuitConf when it's possible
const D: usize = 2;

pub trait Circuit {
    type F: RichField + Extendable<D>;
    type C: GenericConfig<D, F = Self::F>;
    const D: usize = D; // NOTE: Don't override this

    type Targets: ReadPublicInputsTarget;
    type Params;

    fn define(builder: &mut CircuitBuilder<Self::F, D>, params: Self::Params) -> Self::Targets;
    fn build(params: Self::Params) -> (Self::Targets, CircuitData<Self::F, Self::C, D>);

    fn read_public_inputs_target_new(
        public_inputs: &[Target],
    ) -> <Self::Targets as ReadPublicInputsTarget>::PublicInputsTarget {
        Self::Targets::read_public_inputs_target(public_inputs)
    }
}

pub trait ReadPublicInputsTarget {
    type PublicInputsTarget;

    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget;
}

pub trait SerializableCircuit: Circuit {
    fn serialize(targets: &Self::Targets) -> IoResult<Vec<u8>>;
    fn deserialize(data: &mut Buffer) -> IoResult<Self::Targets>;
}

pub trait CircuitWithPublicInputs: Circuit {
    type PublicInputs;
    type PublicInputsTarget;

    fn read_public_inputs(public_inputs: &[Self::F]) -> Self::PublicInputs;
    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget;
}

pub trait WitnessSetter: Circuit {
    type WitnessInput;

    fn set_witness(targets: &Self::Targets, source: &Self::WitnessInput)
        -> PartialWitness<Self::F>;
}
