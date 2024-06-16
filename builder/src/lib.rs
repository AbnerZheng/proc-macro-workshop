use proc_macro::TokenStream;
use proc_macro2::Ident;

use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Field, Fields, FieldsNamed, GenericArgument,
    PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let command_ident = &input.ident;
    let command_builder_ident = format_ident!("{}Builder", command_ident);

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
    let build_method = build_method(fields, command_ident);

    let expand = quote! {
        use std::error::Error;
        pub struct #command_builder_ident {
            #(#fields_def)*
        }

        impl #command_builder_ident {
            #(#setters)*

            #build_method
        }

        impl #command_ident{
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
            if is_option(field) {
                quote! {
                  #name: #ty,
                }
            } else {
                quote! {
                  #name: Option<#ty>,
                }
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
            let ty = if !is_option(field) {
                field.ty.clone()
            } else {
                inner_type_of_option(field)
            };
            quote! {
                 pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                 }
            }
        })
        .collect::<Vec<_>>()
}

fn build_method(fields: &FieldsNamed, command_ident: &Ident) -> proc_macro2::TokenStream {
    let field_check_and_set = fields.named.iter().map(|field| {
        let name = field.ident.as_ref().unwrap();
        let err_msg = format!("field `{}` is missing", name.to_string());

        if is_option(field) {
            quote! {
                #name: self.#name.clone(),
            }
        } else {
            quote! {
                #name: self.#name.as_ref().ok_or(#err_msg)?.clone(),
            }
        }
    });
    quote! {
        pub fn build(&mut self) -> Result<#command_ident, Box<dyn Error>> {
            Ok(#command_ident{
                #(#field_check_and_set)*
            })
        }
    }
}

fn is_option(field: &Field) -> bool {
    let t = &field.ty;
    match t {
        Type::Path(ref type_path) => {
            // eprintln!("{:#?}", type_path);
            type_path.path.segments[0].ident.eq("Option")
        }
        _ => {
            unimplemented!()
        }
    }
}

fn inner_type_of_option(field: &Field) -> Type {
    let t = &field.ty;
    match t {
        Type::Path(ref type_path) => {
            let segment = &type_path.path.segments[0];
            assert!(segment.ident.eq("Option"));
            match segment.arguments {
                PathArguments::AngleBracketed(ref angle) => match angle.args[0] {
                    GenericArgument::Type(ref t) => t.clone(),
                    _ => {
                        unimplemented!()
                    }
                },
                _ => {
                    unimplemented!()
                }
            }
        }
        _ => {
            unimplemented!()
        }
    }
}
