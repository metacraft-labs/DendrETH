use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Field, Fields};

use crate::{
    derive_public_inputs_readable::gen_reader_read,
    utils::{
        create_struct_with_fields, create_struct_with_fields_and_inherited_attrs_target_primitive,
        extend_generics_with_type_param, gen_shorthand_struct_initialization,
        has_functional_attr_with_arg,
    },
};

pub fn impl_derive_circuit_target(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("CircuitTarget is implemented only for structs");
    };

    let targets_with_public_inputs_impl = impl_targets_with_public_inputs(&input_ast, &data);
    let set_witness_impl = impl_set_witness(&input_ast, &data);
    let readable_circuit_input_target_impl = impl_readable_circuit_input_target(&input_ast, &data);

    quote! {
        #targets_with_public_inputs_impl
        #set_witness_impl
        #readable_circuit_input_target_impl
    }
}

fn impl_readable_circuit_input_target(
    input_ast: &DeriveInput,
    struct_data: &DataStruct,
) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;

    let circuit_input_target_ident = format_ident!("{ident}CircuitInputTarget");

    let circuit_input_target_struct_def = create_struct_with_fields(
        &circuit_input_target_ident,
        &input_ast.generics,
        &filter_circuit_input_fields(&struct_data.fields),
    );

    quote! {
        #[derive(circuit_derive::AddVirtualTarget)]
        #circuit_input_target_struct_def

        impl #impl_generics circuit::ReadableCircuitInputTarget for #ident #type_generics #where_clause {
            type CircuitInputTarget = #circuit_input_target_ident #type_generics;

            fn read_circuit_input_target<F: plonky2::hash::hash_types::RichField + plonky2::field::extension::Extendable<D>, const D: usize>(
                builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
            ) -> Self::CircuitInputTarget {
                <Self::CircuitInputTarget as circuit::AddVirtualTarget>::add_virtual_target(builder)
            }
        }
    }
}

fn impl_set_witness(input_ast: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let (_, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;

    let witness_input_ident = format_ident!("{ident}WitnessInput");

    let extended_generics = extend_generics_with_type_param(
        &input_ast.generics,
        quote!(F: plonky2::hash::hash_types::RichField),
    );

    let (extended_impl_generics, _, _) = extended_generics.split_for_impl();

    let circuit_input_fields = filter_circuit_input_fields(&struct_data.fields);

    let witness_input_struct_def = create_struct_with_fields_and_inherited_attrs_target_primitive(
        &witness_input_ident,
        &input_ast.generics,
        &input_ast.attrs,
        &circuit_input_fields,
        &["serde"],
    );

    let set_witness_for_fields = circuit_input_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(<#field_type as circuit::SetWitness<F>>::set_witness(&self.#field_name, witness, &input.#field_name);)
    });

    quote! {
        #witness_input_struct_def

        impl #extended_impl_generics circuit::SetWitness<F> for #ident #type_generics #where_clause {
            type Input = #witness_input_ident #type_generics;

            #[allow(unused_variables)]
            fn set_witness(&self, witness: &mut plonky2::iop::witness::PartialWitness<F>, input: &Self::Input) {
                #(#set_witness_for_fields)*
            }
        }
    }
}

fn impl_targets_with_public_inputs(
    input_ast: &DeriveInput,
    struct_data: &DataStruct,
) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let public_input_fields = filter_public_input_fields(&struct_data.fields);

    let ident = &input_ast.ident;
    let public_inputs_ident = format_ident!("{ident}PublicInputs");
    let public_inputs_target_ident = format_ident!("{ident}PublicInputsTarget");

    let public_inputs_target_struct_def = create_struct_with_fields(
        &public_inputs_target_ident,
        &input_ast.generics,
        &public_input_fields,
    );
    let public_inputs_struct_def = create_struct_with_fields_and_inherited_attrs_target_primitive(
        &public_inputs_ident,
        &input_ast.generics,
        &input_ast.attrs,
        &public_input_fields,
        &["serde"],
    );

    let read_public_inputs = gen_reader_read(&public_input_fields);

    let return_public_inputs_target = gen_shorthand_struct_initialization(
        &public_inputs_target_ident,
        &input_ast.generics,
        &public_input_fields,
    );
    let return_public_inputs = gen_shorthand_struct_initialization(
        &public_inputs_ident,
        &input_ast.generics,
        &public_input_fields,
    );

    let register_public_inputs = public_input_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(builder.register_public_inputs(&<#field_type as circuit::ToTargets>::to_targets(&self.#field_name));)
    });

    // TODO: reuse the PublicInputsReadable macro in some way
    quote! {
        #public_inputs_struct_def
        #public_inputs_target_struct_def

        impl #impl_generics circuit::TargetsWithPublicInputs for #ident #type_generics #where_clause {
            type PublicInputs = #public_inputs_ident #type_generics;

            #[allow(unused_variables)]
            fn read_public_inputs<F: plonky2::hash::hash_types::RichField>(public_inputs: &[F]) -> Self::PublicInputs {
                let mut reader = circuit::PublicInputsFieldReader::new(public_inputs);
                #read_public_inputs
                #return_public_inputs
            }

            type PublicInputsTarget = #public_inputs_target_ident #type_generics;

            #[allow(unused_variables)]
            fn read_public_inputs_target(public_inputs: &[plonky2::iop::target::Target]) -> Self::PublicInputsTarget {
                let mut reader = circuit::PublicInputsTargetReader::new(public_inputs);
                #read_public_inputs
                #return_public_inputs_target
            }

            #[allow(unused_variables)]
            fn register_public_inputs<F: plonky2::hash::hash_types::RichField + plonky2::field::extension::Extendable<D>, const D: usize>(
                &self,
                builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
            ) {
                #(#register_public_inputs)*
            }
        }
    }
}

fn filter_public_input_fields(fields: &Fields) -> Vec<Field> {
    fields
        .into_iter()
        .filter(|&field| has_functional_attr_with_arg(&field, "target", "out"))
        .cloned()
        .collect_vec()
}

fn filter_circuit_input_fields(fields: &Fields) -> Vec<Field> {
    fields
        .into_iter()
        .filter(|&field| has_functional_attr_with_arg(&field, "target", "in"))
        .cloned()
        .collect_vec()
}
