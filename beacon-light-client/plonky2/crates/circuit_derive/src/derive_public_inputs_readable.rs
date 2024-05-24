use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DataStruct, DeriveInput, Field};

use crate::{
    create_struct_with_fields, create_struct_with_fields_and_inherited_attrs_target_primitive,
    gen_shorthand_struct_initialization, utils::concat_token_streams,
};

pub fn impl_derive_public_inputs_readable(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputsReadable is implemented only for structs");
    };

    let public_inputs_target_readable_impl = impl_public_inputs_target_readable(&input_ast, &data);
    let public_inputs_readable_impl = impl_public_inputs_readable(&input_ast, &data);
    let to_targets_impl = impl_to_targets(&input_ast, &data);

    quote! {
        #public_inputs_target_readable_impl
        #public_inputs_readable_impl
        #to_targets_impl-
    }
}

fn impl_public_inputs_target_readable(
    input_ast: &DeriveInput,
    struct_data: &DataStruct,
) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = struct_data.fields.iter().cloned().collect_vec();

    let add_size = struct_data.fields.iter().map(|field| {
        let field_type = &field.ty;
        quote!(size += <#field_type as circuit::PublicInputsTargetReadable>::get_size();)
    });

    let public_inputs_target_struct_def = create_struct_with_fields(
        &format_ident!("{ident}PublicInputsTarget"),
        &input_ast.generics,
        &fields,
    );

    let read_targets = gen_reader_read(&fields);

    let return_from_targets_result =
        gen_shorthand_struct_initialization(&ident, &input_ast.generics, &fields);

    quote! {
        #public_inputs_target_struct_def

        impl #impl_generics circuit::PublicInputsTargetReadable for #ident #type_generics #where_clause {
            fn get_size() -> usize {
                let mut size = 0;
                #(#add_size)*
                size
            }

            fn from_targets(targets: &[plonky2::iop::target::Target]) -> Self {
                assert_eq!(targets.len(), <Self as circuit::PublicInputsTargetReadable>::get_size());
                let mut reader = circuit::PublicInputsTargetReader::new(targets);
                #read_targets
                #return_from_targets_result
            }
        }
    }
}

fn impl_public_inputs_readable(input_ast: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = struct_data.fields.iter().cloned().collect_vec();

    let public_inputs_struct_def = create_struct_with_fields_and_inherited_attrs_target_primitive(
        &format_ident!("{ident}PublicInputs"),
        &input_ast.generics,
        &input_ast.attrs,
        &fields,
        &["serde"],
    );

    let read_field_elements = gen_reader_read(&fields);

    let return_from_elements_result = gen_shorthand_struct_initialization(
        &format_ident!("{ident}Primitive"),
        &input_ast.generics,
        &fields,
    );

    quote! {
        #public_inputs_struct_def

        impl #impl_generics circuit::PublicInputsReadable for #ident #type_generics #where_clause {
            fn from_elements<F: plonky2::hash::hash_types::RichField>(elements: &[F]) -> Self::Primitive {
                assert_eq!(elements.len(), <Self as circuit::PublicInputsTargetReadable>::get_size());
                let mut reader = circuit::PublicInputsFieldReader::new(elements);
                #read_field_elements
                #return_from_elements_result
            }
        }

    }
}

fn impl_to_targets(input_ast: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = struct_data.fields.iter().cloned().collect_vec();

    let push_targets = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_type = &field.ty;
        quote!(targets.extend(<#field_type as circuit::ToTargets>::to_targets(&self.#field_ident));)
    });

    quote! {
        impl #impl_generics circuit::ToTargets for #ident #type_generics #where_clause {
            fn to_targets(&self) -> Vec<plonky2::iop::target::Target> {
                let mut targets = Vec::new();
                #(#push_targets)*
                targets
            }
        }

    }
}

pub fn gen_reader_read(fields: &[Field]) -> TokenStream {
    concat_token_streams(
        fields
            .iter()
            .map(|field| {
                let field_name = &field.ident;
                let field_type = &field.ty;
                // TODO: account for generics (turbofish?)
                quote!(let #field_name = reader.read_object::<#field_type>();)
            })
            .collect_vec(),
    )
}
