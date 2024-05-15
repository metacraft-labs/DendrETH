use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_serde_circuit_target(input_ast: DeriveInput) -> TokenStream {
    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputsReadable is implemented only for structs");
    };

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    let ident = &input_ast.ident;
    let fields = data.fields.iter().cloned().collect_vec();

    let serialize_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(buffer.extend(<#field_type as SerdeCircuitTarget>::serialize(&self.#field_name)?);)
    });

    let deserialize_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote!(#field_name: <#field_type as SerdeCircuitTarget>::deserialize(buffer)?,)
    });

    quote! {
        impl #impl_generics SerdeCircuitTarget for #ident #type_generics #where_clause {
            fn serialize(&self) -> plonky2::util::serialization::IoResult<Vec<u8>> {
                let mut buffer: Vec<u8> = Vec::new();
                #(#serialize_fields)*
                Ok(buffer)
            }

            fn deserialize(
                buffer: &mut plonky2::util::serialization::Buffer
            ) -> plonky2::util::serialization::IoResult<Self> where Self: Sized {
                Ok(Self {
                    #(#deserialize_fields)*
                })
            }
        }
    }
}
