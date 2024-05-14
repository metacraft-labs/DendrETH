use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Fields, Generics, TypeParam};

use crate::utils::{
    concat_token_streams, create_struct_with_fields,
    create_struct_with_fields_and_inherited_attrs_target_primitive,
    gen_shorthand_struct_initialization, has_functional_attr_with_arg,
};

pub fn impl_derive_circuit_target(input_ast: DeriveInput) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = input_ast.generics.split_for_impl();

    let type_param = syn::parse::<TypeParam>(quote!(F: RichField).into()).unwrap();

    let mut modified_generics = input_ast.generics.clone();
    modified_generics
        .params
        .push(syn::GenericParam::Type(type_param));

    let (modified_impl_generics, _, _) = modified_generics.split_for_impl();

    let syn::Data::Struct(ref data) = input_ast.data else {
        panic!("CircuitTarget is implemented only for structs");
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
        create_struct_with_fields_and_inherited_attrs_target_primitive(
            &format_ident!("{ident}PublicInputs"),
            &input_ast.generics,
            &input_ast.attrs,
            &public_input_fields,
            &["serde"],
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
            &input_ast.attrs,
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
}

fn impl_read_public_inputs(input: &DeriveInput, public_input_fields: &[syn::Field]) -> TokenStream {
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
