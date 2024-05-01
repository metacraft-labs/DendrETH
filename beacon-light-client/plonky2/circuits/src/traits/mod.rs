use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{target::Target, witness::PartialWitness},
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitData, config::GenericConfig},
    util::serialization::{Buffer, IoResult},
};

// TODO: stick this in CircuitConf when it's possible
const D: usize = 2;

pub trait CircuitConf {
    type F: RichField + Extendable<D>;
    type C: GenericConfig<D, F = Self::F>;
    const D: usize = D; // NOTE: Don't override this

    type CircuitData = CircuitData<Self::F, Self::C, D>;
}

pub trait Circuit: CircuitConf {
    type Targets;
    type Params;

    fn define(builder: &mut CircuitBuilder<Self::F, D>, params: Self::Params) -> Self::Targets;
    fn build(params: Self::Params) -> (Self::Targets, CircuitData<Self::F, Self::C, D>);
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

pub trait WitnessSetter<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>:
    Circuit
{
    type WitnessInput;

    fn set_witness(targets: &Self::Targets, source: &Self::WitnessInput) -> PartialWitness<F>;
}
