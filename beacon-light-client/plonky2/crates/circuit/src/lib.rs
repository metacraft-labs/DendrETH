#![feature(associated_type_defaults)]
#![feature(array_try_map)]

pub mod add_virtual_target;
pub mod array;
pub mod circuit;
pub mod public_inputs;
pub mod serde_circuit_target;
pub mod set_witness;
pub mod target_primitive;
pub mod to_targets;

pub use add_virtual_target::AddVirtualTarget;
pub use array::Array;
pub use circuit::ReadableCircuitInputTarget;
pub use circuit::{Circuit, SerializableCircuit, TargetsWithPublicInputs};
pub use public_inputs::field_reader::{PublicInputsFieldReader, PublicInputsReadable};
pub use public_inputs::target_reader::{PublicInputsTargetReadable, PublicInputsTargetReader};
pub use serde_circuit_target::SerdeCircuitTarget;
pub use set_witness::SetWitness;
pub use target_primitive::TargetPrimitive;
pub use to_targets::ToTargets;

pub type CircuitInput<T> = <<T as Circuit>::Target as SetWitness<<T as Circuit>::F>>::Input;
pub type CircuitOutput<T> = <<T as Circuit>::Target as TargetsWithPublicInputs>::PublicInputs;
pub type CircuitOutputTarget<T> =
    <<T as Circuit>::Target as TargetsWithPublicInputs>::PublicInputsTarget;
