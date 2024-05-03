use itertools::Itertools;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::format_ident;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Expr;
use syn::Token;
use syn::Type;
use syn::{parse_macro_input, DeriveInput, Field, Fields, Generics, Ident};

#[proc_macro_derive(PublicInputs, attributes(public_input))]
pub fn derive_public_inputs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputs is implemented only for structs");
    };

    let public_input_fields = filter_public_input_fields(&data.fields);

    concat_token_streams(vec![
        impl_read_public_inputs_target(&input_ast, &public_input_fields),
        define_public_inputs_target_struct(&input_ast, &public_input_fields),
    ])
    .into()
}

// struct MacroInput {
//     typ: Type,
//     expr: Expr,
// }
//
// impl Parse for MacroInput {
//     fn parse(input: ParseStream) -> syn::Result<Self> {
//         let typ = input.parse::<Type>()?;
//         let _comma = input.parse::<Token![,]>()?;
//         let expr = input.parse::<Expr>()?;
//         Ok(MacroInput { typ, expr })
//     }
// }
//
// #[proc_macro]
// pub fn read_public_inputs_target(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as MacroInput);
//     let typ = input.typ;
//     let expr = input.expr;
//     quote! {
//         <#typ as Circuit>::Targets::read_public_inputs_target(#expr)
//     }
//     .into()
// }

fn define_public_inputs_target_struct(
    input: &DeriveInput,
    public_input_fields: &[Field],
) -> TokenStream {
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();
    let targets_struct_ident = &input.ident;
    let public_inputs_target_ident = format_ident!("{targets_struct_ident}PublicInputsTarget");

    concat_token_streams(vec![
        quote!(pub struct #public_inputs_target_ident #impl_generics #where_clause),
        enclose_in_braces(gen_public_inputs_target_struct_fields(public_input_fields)),
    ])
}

fn gen_public_inputs_target_struct_fields(public_input_fields: &[Field]) -> TokenStream {
    concat_token_streams(
        public_input_fields
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

fn gen_type_shorthand_initialization(
    type_ident: &Ident,
    generics: &Generics,
    fields: &[Field],
) -> TokenStream {
    let (_, type_generics, _) = generics.split_for_impl();

    let comma_separated_field_names = concat_token_streams(
        fields
            .into_iter()
            .map(|field| {
                let field_ident = &field.ident;
                quote!(#field_ident,)
            })
            .collect_vec(),
    );

    // TODO: :: should not be there if the generics are empty
    concat_token_streams(vec![
        quote!(#type_ident :: #type_generics),
        enclose_in_braces(comma_separated_field_names),
    ])
}

fn impl_read_public_inputs_target(
    input: &DeriveInput,
    public_input_fields: &[Field],
) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let targets_ident = &input.ident;
    let return_ty_ident = format_ident!("{}PublicInputsTarget", input.ident);

    // let signature: TokenStream = quote! {
    //     pub fn read_public_inputs_target #impl_generics(public_inputs: &[Target])
    //         -> #return_ty_ident #type_generics #where_clause
    // };

    let body =
        gen_read_public_inputs_target_body(&return_ty_ident, &input.generics, public_input_fields);

    let trait_impl = quote! {
        impl #impl_generics ReadPublicInputsTarget for #targets_ident #type_generics #where_clause {
            type PublicInputsTarget = #return_ty_ident #type_generics;

            fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget {
                #body
            }
        }
    };

    trait_impl
}

fn gen_read_public_inputs_target_body(
    return_ty_ident: &Ident,
    generics: &Generics,
    public_input_fields: &[Field],
) -> TokenStream {
    concat_token_streams(vec![
        define_reader("PublicInputsTargetReader"),
        gen_public_inputs_target_read_for_fields(public_input_fields),
        gen_type_shorthand_initialization(return_ty_ident, generics, public_input_fields),
    ])
}

fn enclose_in_braces(tokens: TokenStream) -> TokenStream {
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, tokens)))
}

fn concat_token_streams(streams: Vec<TokenStream>) -> TokenStream {
    streams
        .into_iter()
        .fold(TokenStream::new(), |mut concatenated_stream, stream| {
            concatenated_stream.extend(stream);
            concatenated_stream
        })
}

fn define_reader(type_name: &str) -> TokenStream {
    let reader_type_ident = Ident::new(type_name, Span::call_site().into());
    quote!(let mut reader = #reader_type_ident::new(public_inputs);)
}

fn gen_public_inputs_target_read_for_fields(fields: &[Field]) -> TokenStream {
    fields.iter().fold(TokenStream::new(), |mut stream, field| {
        stream.extend(gen_public_inputs_target_read_for_field(field));
        stream
    })
}

fn gen_public_inputs_target_read_for_field(field: &Field) -> TokenStream {
    let field_name = field
        .ident
        .clone()
        .expect("public_input field must be named");

    let field_ty = &field.ty;
    let field_type = quote!(#field_ty);

    quote!(let #field_name = reader.read_object::<#field_type>();)
}

fn filter_public_input_fields(fields: &Fields) -> Vec<Field> {
    fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| {
                attr.path().segments.last().unwrap().ident.to_string() == "public_input"
            })
        })
        .fold(vec![], |mut public_input_fields, field| {
            public_input_fields.push(field.clone());
            public_input_fields
        })
}
