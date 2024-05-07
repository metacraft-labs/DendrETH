use itertools::Itertools;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Fields, Generics, Ident};

#[proc_macro_derive(PublicInputs, attributes(public_input))]
pub fn derive_public_inputs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputs is implemented only for structs");
    };

    let public_input_fields = filter_public_input_fields(&data.fields);

    let read_public_inputs_impl = impl_read_public_inputs(&input_ast, &public_input_fields);
    let read_public_inputs_target_impl =
        impl_read_public_inputs_target(&input_ast, &public_input_fields);
    let register_public_inputs_impl = impl_register_public_inputs(&public_input_fields);

    let ident = &input_ast.ident;

    concat_token_streams(vec![
        define_public_inputs_struct(&input_ast, &public_input_fields),
        create_struct_with_fields(
            &format_ident!("{}PublicInputsTarget", input_ast.ident),
            &input_ast.generics,
            &public_input_fields,
        ),
        quote! {
            impl #impl_generics TargetsWithPublicInputs for #ident #type_generics #where_clause {
                #read_public_inputs_impl
                #read_public_inputs_target_impl
                #register_public_inputs_impl
            }
        },
    ])
    .into()
}

fn create_struct_with_fields(ident: &Ident, generics: &Generics, fields: &[Field]) -> TokenStream {
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let struct_fields = tokenize_struct_fields(fields);

    quote! {
        pub struct #ident #impl_generics #where_clause {
            #struct_fields
        }
    }
}

fn define_public_inputs_struct(input: &DeriveInput, public_input_fields: &[Field]) -> TokenStream {
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();
    let targets_struct_ident = &input.ident;
    let public_inputs_target_ident = format_ident!("{targets_struct_ident}PublicInputs");

    let fields = concat_token_streams(
        public_input_fields
            .into_iter()
            .map(|field| {
                let field_name = &field.ident;
                let target_type = &field.ty;
                let primitive_type = quote!(<#target_type as PublicInputsReadable>::PrimitiveType);
                quote!(#field_name: #primitive_type,)
            })
            .collect_vec(),
    );

    quote! {
        #[derive(Debug)]
        pub struct #public_inputs_target_ident #impl_generics #where_clause {
            #fields
        }

    }
}

fn tokenize_struct_fields(fields: &[Field]) -> TokenStream {
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

fn gen_shorthand_struct_initialization(
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

fn impl_read_public_inputs(input: &DeriveInput, public_input_fields: &[Field]) -> TokenStream {
    let (_, type_generics, _) = input.generics.split_for_impl();

    let targets_ident = &input.ident;
    let return_ty_ident = format_ident!("{targets_ident}PublicInputs");

    let field_readings = concat_token_streams(
        public_input_fields
            .into_iter()
            .map(|field| {
                let field_name = &field.ident;
                let target_type = &field.ty;
                quote!(let #field_name = reader.read_object::<#target_type>();)
            })
            .collect_vec(),
    );

    let return_result =
        gen_shorthand_struct_initialization(&return_ty_ident, &input.generics, public_input_fields);

    quote! {
        type PublicInputs = #return_ty_ident #type_generics;

        fn read_public_inputs<F: RichField>(public_inputs: &[F]) -> Self::PublicInputs {
            let mut reader = PublicInputsFieldReader::new(public_inputs);
            #field_readings
            #return_result
        }
    }
}

fn impl_read_public_inputs_target(
    input: &DeriveInput,
    public_input_fields: &[Field],
) -> TokenStream {
    let (_, type_generics, _) = input.generics.split_for_impl();

    let targets_ident = &input.ident;
    let return_ty_ident = format_ident!("{targets_ident}PublicInputsTarget");

    let body =
        gen_read_public_inputs_target_body(&return_ty_ident, &input.generics, public_input_fields);

    quote! {
        type PublicInputsTarget = #return_ty_ident #type_generics;

        fn read_public_inputs_target(public_inputs: &[Target]) -> Self::PublicInputsTarget {
            #body
        }
    }
}

fn impl_register_public_inputs(public_input_fields: &[Field]) -> TokenStream {
    let register_public_inputs_tokens = concat_token_streams(
        public_input_fields
            .into_iter()
            .map(|field| {
                let field_ident = &field.ident;
                quote!(builder.register_public_inputs(&self.#field_ident.to_targets());)
            })
            .collect_vec(),
    );

    quote! {
        fn register_public_inputs<F: RichField + Extendable<D>, const D: usize>(
            &self,
            builder: &mut CircuitBuilder<F, D>,
        ) {
            #register_public_inputs_tokens
        }
    }
}

fn gen_read_public_inputs_target_body(
    return_ty_ident: &Ident,
    generics: &Generics,
    public_input_fields: &[Field],
) -> TokenStream {
    concat_token_streams(vec![
        define_reader("PublicInputsTargetReader"),
        gen_public_inputs_target_read_for_fields(public_input_fields),
        gen_shorthand_struct_initialization(return_ty_ident, generics, public_input_fields),
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
