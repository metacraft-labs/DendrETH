use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;

use crate::{concat_token_streams, create_struct_with_fields_and_inherited_attrs_target_primitive};

pub fn impl_derive_target_primitive(input_ast: DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("TargetPrimitive is implemented only for structs");
    };

    let ident = &input_ast.ident;

    let primitive_type_ident = format_ident!("{ident}Primitive");

    concat_token_streams(vec![
        create_struct_with_fields_and_inherited_attrs_target_primitive(
            &primitive_type_ident,
            &input_ast.generics,
            &input_ast.attrs,
            data.fields.iter().cloned().collect_vec().as_slice(),
            &["serde"],
        ),
        quote! {
            impl #impl_generics TargetPrimitive for #ident #type_generics #where_clause {
                type Primitive = #primitive_type_ident;
            }
        },
    ])
}
