use itertools::Itertools;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::format_ident;
use quote::quote;
use syn::parse;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Attribute;
use syn::Token;
use syn::TypeParam;
use syn::{parse_macro_input, DeriveInput, Field, Fields, Generics, Ident};

#[derive(Debug)]
struct MetaListValues {
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

#[proc_macro_derive(TargetPrimitive)]
pub fn derive_target_primitive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputs is implemented only for structs");
    };

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let ident = &input_ast.ident;

    let primitive_type_ident = format_ident!("{ident}Primitive");

    concat_token_streams(vec![
        create_struct_with_fields_target_primitive(
            &primitive_type_ident,
            &input_ast.generics,
            data.fields.iter().cloned().collect_vec().as_slice(),
        ),
        quote! {
            impl #impl_generics TargetPrimitive for #ident #type_generics #where_clause {
                type Primitive = #primitive_type_ident;
            }
        }, // quote! {
           //     impl #modified_impl_generics SetWitness<F> for #ident #type_generics #where_clause {
           //         type Input = #witness_input_ident #type_generics;
           //
           //         fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
           //             #set_witness_for_fields
           //         }
           //     }
           // },
    ])
    .into()
}

#[proc_macro_derive(SetWitness, attributes(serde))]
pub fn derive_set_witness(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputs is implemented only for structs");
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

    let witness_input_ident = format_ident!("{ident}WitnessInput");

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
        create_struct_with_fields_and_inherited_attrs_target_primitive(
            &witness_input_ident,
            &input_ast.generics,
            &fields,
            &["serde"],
        ),
        quote! {
            impl #modified_impl_generics SetWitness<F> for #ident #type_generics #where_clause {
                type Input = <#ident #type_generics as TargetPrimitive>::Primitive;

                fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
                    #set_witness_for_fields
                }
            }
        },
    ])
    .into()
}

#[proc_macro_derive(CircuitTarget, attributes(target))]
pub fn derive_public_inputs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();
    // input_ast.generics.params.push(syn::GenericParam::Type())
    let type_param_tokens = quote!(F: RichField);
    let type_param = syn::parse::<TypeParam>(type_param_tokens.into()).unwrap();

    let mut modified_generics = input_ast.generics.clone();
    modified_generics
        .params
        .push(syn::GenericParam::Type(type_param));

    let (modified_impl_generics, _, _) = modified_generics.split_for_impl();

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("PublicInputs is implemented only for structs");
    };

    let public_input_fields = filter_public_input_fields(&data.fields);

    let read_public_inputs_impl = impl_read_public_inputs(&input_ast, &public_input_fields);
    let read_public_inputs_target_impl =
        impl_read_public_inputs_target(&input_ast, &public_input_fields);
    let register_public_inputs_impl = impl_register_public_inputs(&public_input_fields);

    let circuit_input_fields = filter_circuit_input_fields(&data.fields);

    let ident = &input_ast.ident;
    let witness_input_ident = format_ident!("{ident}WitnessInput");

    let set_witness_for_fields = concat_token_streams(
        circuit_input_fields
            .iter()
            .map(|field| {
                let field_name = &field.ident;
                quote!(self.#field_name.set_witness(witness, &input.#field_name);)
            })
            .collect_vec(),
    );

    concat_token_streams(vec![
        create_struct_with_fields_target_primitive(
            &format_ident!("{ident}PublicInputs"),
            &input_ast.generics,
            &public_input_fields,
        ),
        create_struct_with_fields(
            &format_ident!("{ident}PublicInputsTarget"),
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
        create_struct_with_fields_and_inherited_attrs_target_primitive(
            &witness_input_ident,
            &input_ast.generics,
            &circuit_input_fields,
            &["serde"],
        ),
        quote! {
            impl #modified_impl_generics SetWitness<F> for #ident #type_generics #where_clause {
                type Input = #witness_input_ident #type_generics;

                fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
                    #set_witness_for_fields
                }
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

fn create_struct_with_fields_and_inherited_attrs_target_primitive(
    ident: &Ident,
    generics: &Generics,
    fields: &[Field],
    inherited_attrs: &[&str],
) -> TokenStream {
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    let primitive_fields = concat_token_streams(
        fields
            .into_iter()
            .map(|field| {
                let filtered_attrs = field
                    .attrs
                    .iter()
                    .filter(|attr| {
                        inherited_attrs
                            .iter()
                            .any(|inherited_attr| match_attr(attr, inherited_attr))
                    })
                    .collect_vec();

                let filtered_attrs_quoted = concat_token_streams(
                    filtered_attrs
                        .iter()
                        .map(|attr| quote!(#attr))
                        .collect_vec(),
                );

                let field_name = &field.ident;
                let target_type = &field.ty;
                let primitive_type = quote!(<#target_type as TargetPrimitive>::Primitive);
                quote!(#filtered_attrs_quoted pub #field_name: #primitive_type,)
            })
            .collect_vec(),
    );

    quote! {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct #ident #impl_generics #where_clause {
            #primitive_fields
        }

    }
}

fn create_struct_with_fields_target_primitive(
    ident: &Ident,
    generics: &Generics,
    fields: &[Field],
) -> TokenStream {
    create_struct_with_fields_and_inherited_attrs_target_primitive(ident, generics, fields, &[])
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
        .into_iter()
        .filter(|&field| has_functional_attr_with_arg(&field, "target", "out"))
        .cloned()
        .collect_vec()
}

fn filter_circuit_input_fields(fields: &Fields) -> Vec<Field> {
    fields
        .into_iter()
        .filter(|&field| has_functional_attr_with_arg(&field, "target", "in"))
        .cloned()
        .collect_vec()
}

fn has_functional_attr_with_arg(field: &Field, attr: &str, arg: &str) -> bool {
    if let Some(attr) = find_attr(&field.attrs, attr) {
        if let syn::Meta::List(list) = &attr.meta {
            let items = parse::<MetaListValues>(list.tokens.clone().into()).unwrap();
            if items.values.into_iter().any(|attr| attr == arg) {
                return true;
            }
        }
    }
    false
}

// fn attrs_contain(attrs: &[Attribute], attribute: &str) -> bool {
//     attrs.into_iter().any(|attr| match_attr(attr, attribute))
// }

fn match_attr(attr: &Attribute, string: &str) -> bool {
    attr.path().segments.last().unwrap().ident.to_string() == string
}

fn find_attr<'a>(attrs: &'a [Attribute], attr: &str) -> Option<&'a Attribute> {
    attrs
        .into_iter()
        .find(|&attribute| match_attr(attribute, attr))
}
