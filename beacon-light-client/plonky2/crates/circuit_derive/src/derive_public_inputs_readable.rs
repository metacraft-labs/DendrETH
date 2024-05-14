use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, TypeParam};

use crate::{
    create_struct_with_fields, create_struct_with_fields_and_inherited_attrs_target_primitive,
    gen_shorthand_struct_initialization,
};

pub fn impl_derive_public_inputs_readable(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputsReadable is implemented only for structs");
    };

    let fields = data.fields.iter().cloned().collect_vec();

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let type_param_tokens = quote!(F: RichField);
    let type_param = syn::parse::<TypeParam>(type_param_tokens.into()).unwrap();

    let mut modified_generics = input_ast.generics.clone();
    modified_generics
        .params
        .push(syn::GenericParam::Type(type_param));

    let ident = &input_ast.ident;

    let add_size = fields.iter().map(|field| {
        let field_type = &field.ty;
        quote!(size += <#field_type as PublicInputsTargetReadable>::get_size();)
    });

    let read_targets = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        // TODO: account for generics (turbofish)
        quote!(let #field_name = reader.read_object::<#field_type>();)
    });

    let read_field_elements = read_targets.clone();

    let return_from_targets_result =
        gen_shorthand_struct_initialization(&ident, &input_ast.generics, &fields);

    let return_from_elements_result = gen_shorthand_struct_initialization(
        &format_ident!("{ident}Primitive"),
        &input_ast.generics,
        &fields,
    );

    let push_targets = fields.iter().map(|field| {
        let field_ident = &field.ident;
        quote!(targets.extend(self.#field_ident.to_targets());)
    });

    let public_inputs_target_struct_def = create_struct_with_fields(
        &format_ident!("{ident}PublicInputsTarget"),
        &input_ast.generics,
        &fields,
    );

    let public_inputs_struct_def = create_struct_with_fields_and_inherited_attrs_target_primitive(
        &format_ident!("{ident}PublicInputs"),
        &input_ast.generics,
        &input_ast.attrs,
        &fields,
        &["serde"],
    );

    quote! {
        #public_inputs_target_struct_def
        #public_inputs_struct_def

        impl #impl_generics PublicInputsTargetReadable for #ident #type_generics #where_clause {
            fn get_size() -> usize {
                let mut size = 0;
                #(#add_size)*
                size
            }

            fn from_targets(targets: &[Target]) -> Self {
                assert_eq!(targets.len(), Self::get_size());
                let mut reader = PublicInputsTargetReader::new(targets);
                #(#read_targets)*
                #return_from_targets_result
            }
        }

        impl #impl_generics PublicInputsReadable for #ident #type_generics #where_clause {
            fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
                assert_eq!(elements.len(), Self::get_size());
                let mut reader = PublicInputsFieldReader::new(elements);
                #(#read_field_elements)*
                #return_from_elements_result
            }
        }

        impl #impl_generics ToTargets for #ident #type_generics #where_clause {
            fn to_targets(&self) -> Vec<Target> {
                let mut targets = Vec::new();
                #(#push_targets)*
                targets
            }
        }
    }
}
