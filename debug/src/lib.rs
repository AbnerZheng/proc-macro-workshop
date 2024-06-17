use proc_macro::TokenStream;

use quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input};

#[proc_macro_derive(CustomDebug)]
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
                let ident_str = ident.to_string();
                Ok(quote! {
                    .field(#ident_str, &self.#ident)
                })
            })
            .collect::<syn::Result<Vec<_>>>(),
        _ => {
            unimplemented!()
        }
    }?;

    let token = quote! {
        impl std::fmt::Debug for #struct_ident{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#struct_ident_str)
                 #(#fields)*
                 .finish()
            }
        }
    };
    Ok(token)
}
