use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::utils::extend_generics_with_type_param;

pub fn impl_derive_set_witness(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("SetWitness is implemented only for structs");
    };

    let (_, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = data.fields.iter().cloned().collect_vec();

    let extended_generics =
        extend_generics_with_type_param(&input_ast.generics, quote!(F: RichField));
    let (extended_impl_generics, _, _) = extended_generics.split_for_impl();

    let set_witness_for_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(<#field_type as circuit::SetWitness<F>>::set_witness(&self.#field_name, witness, &input.#field_name);)
    });

    quote! {
        impl #extended_impl_generics circuit::SetWitness<F> for #ident #type_generics #where_clause {
            type Input = <#ident #type_generics as circuit::TargetPrimitive>::Primitive;

            fn set_witness(&self, witness: &mut plonky2::iop::witness::PartialWitness<F>, input: &Self::Input) {
                #(#set_witness_for_fields)*
            }
        }
    }
}
