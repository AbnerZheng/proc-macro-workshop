use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let input_ident = &input.ident;
    let command_builder_ident = format_ident!("{}Builder", input_ident);

    // eprintln!("{:#?}", input.attrs);
    // eprintln!("{:#?}", input.data);
    // eprintln!("{:#?}", input.span());

    let fields: (Vec<_>, Vec<_>) = match input.data {
        Data::Struct(ref data) => {
            // eprintln!("fields: {:#?}", data.fields);
            // eprintln!("semi_token: {:#?}", data.semi_token);
            // eprintln!("struct_token: {:#?}", data.struct_token);
            match data.fields {
                Fields::Unnamed(_) | Fields::Unit => {
                    unimplemented!()
                }
                Fields::Named(ref fields) => fields
                    .named
                    .iter()
                    .map(|field| {
                        let name = field.ident.as_ref().unwrap();
                        let ty = &field.ty;
                        let def = quote! {
                          #name: Option<#ty>,
                        };
                        let default_value = quote! {
                            #name: None,
                        };
                        (def, default_value)
                    })
                    .collect::<Vec<_>>()
                    .iter()
                    .cloned()
                    .unzip(),
            }
        }
        Data::Enum(_) | Data::Union(_) => {
            unimplemented!()
        }
    };

    let fields_def = fields.0;
    let fields_default_value = fields.1;

    let expand = quote! {
        pub struct #command_builder_ident{
            #(#fields_def)*
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
