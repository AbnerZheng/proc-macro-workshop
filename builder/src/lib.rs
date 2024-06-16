use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, FieldsNamed};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_ident = &input.ident;
    let command_builder_ident = format_ident!("{}Builder", input_ident);

    // eprintln!("{:#?}", input.attrs);
    // eprintln!("{:#?}", input.data);
    // eprintln!("{:#?}", input.span());

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Unnamed(_) | Fields::Unit => {
                unimplemented!()
            }
            Fields::Named(ref fields) => fields,
        },
        Data::Enum(_) | Data::Union(_) => {
            unimplemented!()
        }
    };

    let setters = setter_methods(fields);
    let fields_def = fields_definitions(fields);
    let fields_default_value = fields_default_values(fields);

    let expand = quote! {
        pub struct #command_builder_ident {
            #(#fields_def)*
        }

        impl #command_builder_ident {
            #(#setters)*
        }

        impl #input_ident{
            pub fn builder() -> #command_builder_ident{
                let builder = #command_builder_ident{
                    #(#fields_default_value)*
                };
                builder
            }
        }
    };
    TokenStream::from(expand)
}

fn fields_default_values(fields_named: &FieldsNamed) -> Vec<proc_macro2::TokenStream> {
    fields_named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                #name: None,
            }
        })
        .collect()
}

fn fields_definitions(fields_named: &FieldsNamed) -> Vec<proc_macro2::TokenStream> {
    fields_named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            quote! {
                #name: Option<#ty>,
            }
        })
        .collect()
}

fn setter_methods(fields: &FieldsNamed) -> Vec<proc_macro2::TokenStream> {
    fields
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            quote! {
                 fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                 }
            }
        })
        .collect::<Vec<_>>()
}
