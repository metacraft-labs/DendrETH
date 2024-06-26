use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::gen_shorthand_struct_initialization;

pub fn impl_derive_add_virtual_target(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("AddVirtualTarget is implemented only for structs");
    };

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = data.fields.iter().cloned().collect_vec();

    let add_virtual_targets = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(let #field_name = <#field_type as circuit::AddVirtualTarget>::add_virtual_target(builder);)
    });

    let return_result = gen_shorthand_struct_initialization(&ident, &input_ast.generics, &fields);

    quote! {
        impl #impl_generics circuit::AddVirtualTarget for #ident #type_generics #where_clause {
            fn add_virtual_target<F: plonky2::hash::hash_types::RichField + plonky2::field::extension::Extendable<D>, const D: usize>(
                builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
            ) -> Self {
                #(#add_virtual_targets)*
                #return_result
            }
        }
    }
}
