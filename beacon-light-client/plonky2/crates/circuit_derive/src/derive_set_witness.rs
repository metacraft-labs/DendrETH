use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, TypeParam};

use crate::utils::concat_token_streams;

pub fn impl_derive_set_witness(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("SetWitness is implemented only for structs");
    };

    let fields = data.fields.iter().cloned().collect_vec();

    let (_, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let type_param_tokens = quote!(F: RichField);
    let type_param = syn::parse::<TypeParam>(type_param_tokens.into()).unwrap();

    let mut modified_generics = input_ast.generics.clone();
    modified_generics
        .params
        .push(syn::GenericParam::Type(type_param));

    let (modified_impl_generics, _, _) = modified_generics.split_for_impl();

    let ident = &input_ast.ident;

    let set_witness_for_fields = concat_token_streams(
        fields
            .iter()
            .map(|field| {
                let field_name = &field.ident;
                quote!(self.#field_name.set_witness(witness, &input.#field_name);)
            })
            .collect_vec(),
    );

    concat_token_streams(vec![
        // create_struct_with_fields_and_inherited_attrs_target_primitive(
        //     &witness_input_ident,
        //     &input_ast.generics,
        //     &fields,
        //     &["serde"],
        // ),
        quote! {
            impl #modified_impl_generics SetWitness<F> for #ident #type_generics #where_clause {
                type Input = <#ident #type_generics as TargetPrimitive>::Primitive;

                fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
                    #set_witness_for_fields
                }
            }
        },
    ])
}
