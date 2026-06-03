use std::array;

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn srgb(tokens: TokenStream) -> TokenStream {
    fn inner(lit: LitStr) -> syn::Result<proc_macro2::TokenStream> {
        let string = lit.value();

        let hex_str = string.strip_prefix('#').unwrap_or(&string);

        let err = syn::Error::new_spanned(
            &lit,
            "expected 3-byte color (i.e. \"#FFFFFF\" or \"FFFFFF\")",
        );

        let [r, g, b] = array::from_fn(|i| {
            u8::from_str_radix(hex_str.get(i * 2..(i + 1) * 2).ok_or(err.clone())?, 16)
                .map_err(|e| syn::Error::new_spanned(&lit, e.to_string()))
        });

        let [r, g, b] = [r?, g?, b?];

        if hex_str.len() > 6 {
            Err(err)?
        }

        Ok(quote! {
            ::colors::Srgb8 { r: #r, g: #g, b: #b }
        })
    }

    match inner(parse_macro_input!(tokens as LitStr)) {
        Ok(out) => out,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
