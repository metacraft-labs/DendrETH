use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Field, Generics, Token, TypeParam,
};

#[derive(Debug)]
pub struct MetaListValues {
    pub values: Vec<String>,
}

impl Parse for MetaListValues {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut values = Vec::new();

        let mut is_first_value = true;

        while !input.is_empty() {
            if !is_first_value {
                let _comma: Token![,] = input.parse()?;
            }
            is_first_value = false;

            let string: String = match input.parse::<Ident>() {
                Ok(ident) => ident.to_string(),
                Err(_) => match input.parse::<Token![in]>() {
                    Ok(_) => "in".to_string(),
                    Err(_) => panic!("Input item to targets() must be ident or in"),
                },
            };
            values.push(string);
        }
        Ok(MetaListValues { values })
    }
}
pub fn has_functional_attr_with_arg(field: &Field, attr: &str, arg: &str) -> bool {
    if let Some(attr) = find_attr(&field.attrs, attr) {
        if let syn::Meta::List(list) = &attr.meta {
            let items = syn::parse::<MetaListValues>(list.tokens.clone().into()).unwrap();
            if items.values.into_iter().any(|attr| attr == arg) {
                return true;
            }
        }
    }
    false
}

pub fn match_attr(attr: &Attribute, string: &str) -> bool {
    attr.path().segments.last().unwrap().ident.to_string() == string
}

pub fn find_attr<'a>(attrs: &'a [Attribute], attr: &str) -> Option<&'a Attribute> {
    attrs
        .into_iter()
        .find(|&attribute| match_attr(attribute, attr))
}

#[allow(dead_code)]
pub fn create_struct_with_fields_target_primitive(
    ident: &Ident,
    generics: &Generics,
    fields: &[Field],
) -> TokenStream {
    create_struct_with_fields_and_inherited_attrs_target_primitive(
        ident,
        generics,
        &[],
        fields,
        &[],
    )
}

pub fn create_struct_with_fields(
    ident: &Ident,
    generics: &Generics,
    fields: &[Field],
) -> TokenStream {
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let struct_fields = tokenize_struct_fields(fields);

    quote! {
        pub struct #ident #impl_generics #where_clause {
            #struct_fields
        }
    }
}

pub fn filter_attrs(attrs: &[Attribute], filter_set: &[&str]) -> Vec<Attribute> {
    attrs
        .iter()
        .cloned()
        .filter(|attr| {
            filter_set
                .iter()
                .any(|inherited_attr| match_attr(attr, inherited_attr))
        })
        .collect_vec()
}

pub fn create_struct_with_fields_and_inherited_attrs_target_primitive(
    ident: &Ident,
    generics: &Generics,
    attrs: &[Attribute],
    fields: &[Field],
    inherited_attrs: &[&str],
) -> TokenStream {
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let inherited_struct_attrs = filter_attrs(&attrs, inherited_attrs);
    let inherited_struct_attrs_tokens = inherited_struct_attrs.iter().map(|attr| quote!(#attr));

    let primitive_fields = fields.into_iter().map(|field| {
        let field_name = &field.ident;
        let target_type = &field.ty;
        let primitive_type = quote!(<#target_type as TargetPrimitive>::Primitive);

        let inherited_attrs = filter_attrs(&field.attrs, inherited_attrs);
        let attr_tokens = inherited_attrs.iter().map(|attr| quote!(#attr));

        quote!(#(#attr_tokens)* pub #field_name: #primitive_type,)
    });

    quote! {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #(#inherited_struct_attrs_tokens)*
        pub struct #ident #impl_generics #where_clause {
            #(#primitive_fields)*
        }
    }
}

pub fn tokenize_struct_fields(fields: &[Field]) -> TokenStream {
    concat_token_streams(
        fields
            .into_iter()
            .map(|field| {
                let field_ty = &field.ty;
                let field_type = quote!(#field_ty);
                let field_name = &field.ident;
                quote!(pub #field_name: #field_type,)
            })
            .collect_vec(),
    )
}

pub fn gen_shorthand_struct_initialization(
    type_ident: &Ident,
    generics: &Generics,
    fields: &[Field],
) -> TokenStream {
    let (_, type_generics, _) = generics.split_for_impl();

    let comma_separated_field_names = list_struct_fields(fields);
    let turbofish_type_generics = type_generics.as_turbofish();

    quote! {
        #type_ident #turbofish_type_generics {
            #(#comma_separated_field_names)*
        }
    }
}

pub fn concat_token_streams(streams: Vec<TokenStream>) -> TokenStream {
    streams
        .into_iter()
        .fold(TokenStream::new(), |mut concatenated_stream, stream| {
            concatenated_stream.extend(stream);
            concatenated_stream
        })
}

pub fn extend_generics_with_type_param(
    generics: &Generics,
    generic_bound: TokenStream,
) -> Generics {
    let type_param = syn::parse::<TypeParam>(generic_bound.into()).unwrap();

    let mut extended_generics = generics.clone();
    extended_generics
        .params
        .push(syn::GenericParam::Type(type_param));

    extended_generics
}

pub fn list_struct_fields(fields: &[Field]) -> Vec<TokenStream> {
    fields
        .into_iter()
        .map(|field| {
            let field_ident = &field.ident;
            quote!(#field_ident,)
        })
        .collect_vec()
}
