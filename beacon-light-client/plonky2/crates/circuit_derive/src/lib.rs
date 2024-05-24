use derive_add_virtual_target::impl_derive_add_virtual_target;
use derive_circuit_target::impl_derive_circuit_target;
use derive_public_inputs_readable::impl_derive_public_inputs_readable;
use derive_serde_circuit_target::impl_serde_circuit_target;
use derive_set_witness::impl_derive_set_witness;
use derive_target_primitive::impl_derive_target_primitive;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::DeriveInput;
use utils::create_struct_with_fields;
use utils::create_struct_with_fields_and_inherited_attrs_target_primitive;
use utils::gen_shorthand_struct_initialization;

mod derive_add_virtual_target;
mod derive_circuit_target;
mod derive_public_inputs_readable;
mod derive_serde_circuit_target;
mod derive_set_witness;
mod derive_target_primitive;
mod utils;

#[proc_macro_derive(TargetPrimitive)]
pub fn derive_target_primitive(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_derive_target_primitive(input_ast).into()
}

#[proc_macro_derive(AddVirtualTarget)]
pub fn derive_add_virtual_target(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_derive_add_virtual_target(input_ast).into()
}

#[proc_macro_derive(PublicInputsReadable, attributes(serde))]
pub fn derive_public_inputs_readable(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_derive_public_inputs_readable(input_ast).into()
}

#[proc_macro_derive(CircuitTarget, attributes(target, serde))]
pub fn derive_circuit_target(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_derive_circuit_target(input_ast).into()
}

#[proc_macro_derive(SetWitness, attributes(serde))]
pub fn derive_set_witness(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_derive_set_witness(input_ast).into()
}

#[proc_macro_derive(SerdeCircuitTarget)]
pub fn derive_serde_circuit_target(input: TokenStream) -> TokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);
    impl_serde_circuit_target(input_ast).into()
}
