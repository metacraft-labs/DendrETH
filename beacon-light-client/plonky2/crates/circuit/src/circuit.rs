use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::Target,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::GenericConfig,
    },
};

use crate::{CircuitInputTarget, CircuitOutput, CircuitOutputTarget};

// TODO: stick D in the circuit config when const generics mature enough
// this D's value is the same as GoldilocksPoseidonConfig's D
const D: usize = 2;

pub trait Circuit {
    type F: RichField + Extendable<D>;
    type C: GenericConfig<D, F = Self::F>;
    const D: usize = D; // NOTE: Don't override this
    const CIRCUIT_CONFIG: CircuitConfig;

    type Target: TargetsWithPublicInputs + ReadableCircuitInputTarget;
    type Params;

    fn define(builder: &mut CircuitBuilder<Self::F, D>, params: &Self::Params) -> Self::Target;

    fn build(params: &Self::Params) -> (Self::Target, CircuitData<Self::F, Self::C, D>) {
        let mut builder = CircuitBuilder::new(Self::CIRCUIT_CONFIG);
        let targets = Self::define(&mut builder, params);
        targets.register_public_inputs(&mut builder);

        let circuit_data = builder.build::<Self::C>();
        (targets, circuit_data)
    }

    fn read_public_inputs_target(public_inputs: &[Target]) -> CircuitOutputTarget<Self> {
        Self::Target::read_public_inputs_target(public_inputs)
    }

    fn read_public_inputs(public_inputs: &[Self::F]) -> CircuitOutput<Self> {
        Self::Target::read_public_inputs(public_inputs)
    }

    fn read_circuit_input_target(
        builder: &mut CircuitBuilder<Self::F, D>,
    ) -> CircuitInputTarget<Self> {
        Self::Target::read_circuit_input_target(builder)
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

pub trait ReadableCircuitInputTarget {
    type CircuitInputTarget;

    fn read_circuit_input_target<F: RichField + Extendable<D>, const D: usize>(
        builder: &mut CircuitBuilder<F, D>,
    ) -> Self::CircuitInputTarget;
}
