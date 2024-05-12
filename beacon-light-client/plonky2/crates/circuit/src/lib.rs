#![feature(associated_type_defaults)]

pub mod array;
pub mod public_inputs;
pub mod set_witness;
pub mod target_primitive;
pub mod to_targets;

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::GenericConfig,
    },
    util::serialization::{Buffer, IoResult},
};
use set_witness::SetWitness;

// TODO: stick this in CircuitConf when it's possible
const D: usize = 2;

pub trait Circuit {
    type F: RichField + Extendable<D>;
    type C: GenericConfig<D, F = Self::F>;
    const D: usize = D; // NOTE: Don't override this
    const CIRCUIT_CONFIG: CircuitConfig;

    type Targets: TargetsWithPublicInputs;
    type Params;

    fn define(builder: &mut CircuitBuilder<Self::F, D>, params: Self::Params) -> Self::Targets;
    fn build(params: Self::Params) -> (Self::Targets, CircuitData<Self::F, Self::C, D>) {
        let mut builder = CircuitBuilder::new(Self::CIRCUIT_CONFIG);
        let targets = Self::define(&mut builder, params);
        targets.register_public_inputs(&mut builder);

        let circuit_data = builder.build::<Self::C>();
        (targets, circuit_data)
    }

    fn read_public_inputs_target(
        public_inputs: &[Target],
    ) -> <Self::Targets as TargetsWithPublicInputs>::PublicInputsTarget {
        Self::Targets::read_public_inputs_target(public_inputs)
    }

    fn read_public_inputs(
        public_inputs: &[Self::F],
    ) -> <Self::Targets as TargetsWithPublicInputs>::PublicInputs {
        Self::Targets::read_public_inputs(public_inputs)
    }
}

pub trait TargetsWithPublicInputs {
    type PublicInputsTarget;
    type PublicInputs;

    fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget;
    fn read_public_inputs<F: RichField>(public_inputs: &[F]) -> Self::PublicInputs;
    fn register_public_inputs<F: RichField + Extendable<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<F, D>,
    );
}

pub trait SerializableCircuit: Circuit {
    fn serialize(targets: &Self::Targets) -> IoResult<Vec<u8>>;
    fn deserialize(data: &mut Buffer) -> IoResult<Self::Targets>;
}

pub type CircuitInput<T> = <<T as Circuit>::Targets as SetWitness<<T as Circuit>::F>>::Input;
pub type CircuitPublicInputs<T> =
    <<T as Circuit>::Targets as TargetsWithPublicInputs>::PublicInputs;
