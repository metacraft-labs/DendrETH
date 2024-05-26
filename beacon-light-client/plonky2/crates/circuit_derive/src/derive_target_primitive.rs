use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::create_struct_with_fields_and_inherited_attrs_target_primitive;

pub fn impl_derive_target_primitive(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("TargetPrimitive is implemented only for structs");
    };

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;

    let primitive_type_ident = format_ident!("{ident}Primitive");

    let primitive_type_struct_def = create_struct_with_fields_and_inherited_attrs_target_primitive(
        &primitive_type_ident,
        &input_ast.generics,
        &input_ast.attrs,
        data.fields.iter().cloned().collect_vec().as_slice(),
        &["serde"],
    );

    quote! {
        #primitive_type_struct_def

        impl #impl_generics circuit::TargetPrimitive for #ident #type_generics #where_clause {
            type Primitive = #primitive_type_ident;
        }
    }
}
