#![feature(associated_type_defaults)]
#![feature(array_try_map)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod add_virtual_target;
pub mod array;
pub mod circuit;
pub mod circuit_builder_extensions;
pub mod public_inputs;
pub mod serde_circuit_target;
pub mod set_witness;
pub mod target_primitive;
pub mod to_targets;

pub use add_virtual_target::AddVirtualTarget;
pub use array::Array;
pub use circuit::{Circuit, ReadableCircuitInputTarget, ReadablePublicInputs};
use plonky2::plonk::circuit_data::CircuitData;
pub use public_inputs::{
    field_reader::{PublicInputsFieldReader, PublicInputsReadable},
    target_reader::{PublicInputsTargetReadable, PublicInputsTargetReader},
};
pub use serde_circuit_target::SerdeCircuitTarget;
pub use set_witness::SetWitness;
pub use target_primitive::TargetPrimitive;
pub use to_targets::ToTargets;

pub type CircuitInput<T> = <<T as Circuit>::Target as SetWitness<<T as Circuit>::F>>::Input;
pub type CircuitInputTarget<T> =
    <<T as Circuit>::Target as ReadableCircuitInputTarget>::CircuitInputTarget;
pub type CircuitOutput<T> = <<T as Circuit>::Target as ReadablePublicInputs>::PublicInputs;
pub type CircuitOutputTarget<T> =
    <<T as Circuit>::Target as ReadablePublicInputs>::PublicInputsTarget;
pub type CircuitTargetType<T> = <T as Circuit>::Target;

#[allow(type_alias_bounds)]
pub type CircuitDataType<T: Circuit> =
    CircuitData<<T as Circuit>::F, <T as Circuit>::C, { <T as Circuit>::D }>;
