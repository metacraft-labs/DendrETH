use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{target::Target, witness::PartialWitness},
    plonk::{circuit_builder::CircuitBuilder, circuit_data::CircuitData, config::GenericConfig},
    util::serialization::{Buffer, IoResult},
};

pub trait Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    type Targets;
    type Params;

    fn define(builder: &mut CircuitBuilder<F, D>, params: Self::Params) -> Self::Targets;

    fn build(params: Self::Params) -> (Self::Targets, CircuitData<F, C, D>);
}

pub trait SerializableCircuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>: Circuit<F, C, D>
{
    fn serialize(targets: &Self::Targets) -> IoResult<Vec<u8>>;
    fn deserialize(data: &mut Buffer) -> IoResult<Self::Targets>;
}

pub trait CircuitWithPublicInputs<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
>: Circuit<F, C, D>
{
    type PublicInputs;
    type PublicInputsTarget;

    fn read_public_inputs(public_inputs: &[F]) -> Self::PublicInputs;

    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget;
}

pub trait WitnessSetter<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>:
    Circuit<F, C, D>
{
    type WitnessInput;

    fn set_witness(targets: &Self::Targets, source: &Self::WitnessInput) -> PartialWitness<F>;
}
