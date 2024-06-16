use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Field, Fields, FieldsNamed, GenericArgument,
    LitStr, PathArguments, Result, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let command_ident = &input.ident;
    let command_builder_ident = format_ident!("{}Builder", command_ident);

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

    let setters = setter_methods(fields)?;
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
    Ok(expand)
}

fn fields_default_values(fields_named: &FieldsNamed) -> Vec<proc_macro2::TokenStream> {
    fields_named
        .named
        .iter()
        .map(|field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                #name: std::option::Option::None,
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
                  #name: std::option::Option<#ty>,
                }
            }
        })
        .collect()
}

fn setter_methods(fields: &FieldsNamed) -> Result<Vec<proc_macro2::TokenStream>> {
    let mut methods = vec![];
    for field in fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = if !is_option(field) {
            field.ty.clone()
        } else {
            inner_type_of_option(field)
        };
        // if the filed has attribute `build`, we had add a one_at_once for this field
        let mut each_ident: Option<String> = None;
        for attr in &field.attrs {
            if attr.path().is_ident("builder") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("each") {
                        let value = meta.value()?;
                        let liter: LitStr = value.parse()?;
                        each_ident = Some(liter.value());
                        Ok(())
                    } else {
                        Err(Error::new_spanned(
                            attr,
                            "expected `#[builder(each = \"...\")]`",
                        ))
                    }
                })?;
            }
        }
        let method = match each_ident {
            Some(ref v) if *v == name.to_string() => {
                let v = format_ident!("{}", v);
                let inner_ty = inner_type_of_vec(field);

                quote! {
                    pub fn #v(&mut self, #name: #inner_ty) -> &mut Self {
                        self.#name.get_or_insert(vec![]).push(#name);
                        self
                    }
                }
            }
            Some(ref v) if *v != name.to_string() => {
                let v = format_ident!("{}", v);
                let inner_ty = inner_type_of_vec(field);

                quote! {
                     pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = std::option::Option::Some(#name);
                        self
                     }

                    pub fn #v(&mut self, #v: #inner_ty) -> &mut Self {
                        self.#name.get_or_insert(vec![]).push(#v);
                        self
                    }
                }
            }

            _ => {
                quote! {
                     pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = std::option::Option::Some(#name);
                        self
                     }
                }
            }
        };
        methods.push(method);
    }
    Ok(methods)
}

fn inner_type_of_vec(field: &Field) -> Type {
    let t = &field.ty;
    match t {
        Type::Path(ref type_path) => {
            let segment = &type_path.path.segments[0];
            assert!(segment.ident.eq("Vec"));
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
        pub fn build(&mut self) -> std::result::Result<#command_ident, std::boxed::Box<dyn Error>> {
            std::result::Result::Ok(#command_ident{
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
