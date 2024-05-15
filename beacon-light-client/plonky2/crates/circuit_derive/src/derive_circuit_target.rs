use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Field, Fields};

use crate::utils::{
    create_struct_with_fields, create_struct_with_fields_and_inherited_attrs_target_primitive,
    extend_generics_with_type_param, gen_shorthand_struct_initialization,
    has_functional_attr_with_arg,
};

pub fn impl_derive_circuit_target(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("CircuitTarget is implemented only for structs");
    };

    let set_witness_impl = impl_set_witness(&input_ast, &data);
    let targets_with_public_inputs_impl = impl_targets_with_public_inputs(&input_ast, &data);

    quote! {
        #targets_with_public_inputs_impl
        #set_witness_impl
    }
}

fn impl_set_witness(input_ast: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let (_, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let ident = &input_ast.ident;
    let witness_input_ident = format_ident!("{ident}WitnessInput");

    let extended_generics =
        extend_generics_with_type_param(&input_ast.generics, quote!(F: RichField));
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
        quote!(self.#field_name.set_witness(witness, &input.#field_name);)
    });

    quote! {
        #witness_input_struct_def

        impl #extended_impl_generics SetWitness<F> for #ident #type_generics #where_clause {
            type Input = #witness_input_ident #type_generics;

            fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
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

    let read_public_inputs_targets = public_input_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(let #field_name = reader.read_object::<#field_type>();)
    });
    let read_public_inputs = read_public_inputs_targets.clone();

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
        let field_ident = &field.ident;
        quote!(builder.register_public_inputs(&self.#field_ident.to_targets());)
    });

    quote! {
        #public_inputs_struct_def
        #public_inputs_target_struct_def

        impl #impl_generics TargetsWithPublicInputs for #ident #type_generics #where_clause {
            type PublicInputs = #public_inputs_ident #type_generics;

            fn read_public_inputs<F: RichField>(public_inputs: &[F]) -> Self::PublicInputs {
                let mut reader = PublicInputsFieldReader::new(public_inputs);
                #(#read_public_inputs)*
                #return_public_inputs
            }

            type PublicInputsTarget = #public_inputs_target_ident #type_generics;

            fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget {
                let mut reader = PublicInputsTargetReader::new(public_inputs);
                #(#read_public_inputs_targets)*
                #return_public_inputs_target
            }

            fn register_public_inputs<F: RichField + Extendable<D>, const D: usize>(
                &self,
                builder: &mut CircuitBuilder<F, D>,
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
