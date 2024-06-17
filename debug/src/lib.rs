use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Expr, Field};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = &input.ident;
    let struct_ident_str = struct_ident.to_string();

    let fields = match &input.data {
        Data::Struct(ref data) => data
            .fields
            .iter()
            .map(|field| {
                let ident = field
                    .ident
                    .as_ref()
                    .ok_or(Error::new_spanned(field, "Expected named struct"))?;
                let debug_pattern = debug_attr(field)?;
                let ident_str = ident.to_string();
                match debug_pattern {
                    None => Ok(quote! {
                        .field(#ident_str, &self.#ident)
                    }),
                    Some(pattern) => Ok(quote! {
                        .field(#ident_str, &format_args!(#pattern, self.#ident))
                    }),
                }
            })
            .collect::<syn::Result<Vec<_>>>(),
        _ => {
            unimplemented!()
        }
    }?;

    let generics = &input.generics;
    let type_params = generics.type_params();
    let type_params = type_params
        .map(|t| {
            quote! {
                #t: std::fmt::Debug
            }
        })
        .collect::<Vec<_>>();
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl <#(#type_params),*> std::fmt::Debug for #struct_ident #ty_generics #where_clause{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_ident_str)
                 #(#fields)*
                 .finish()
            }
        }
    })
}

fn debug_attr(field: &Field) -> syn::Result<Option<Expr>> {
    let mut format_pattern = None;
    for attr in &field.attrs {
        if attr.path().is_ident("debug") {
            let name_value = &attr.meta.require_name_value()?.value;
            format_pattern = Some(name_value.clone());
        } else {
            return Err(syn::Error::new_spanned(attr, "expect `debug=\"...\"`"));
        }
    }
    Ok(format_pattern)
}
