use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::guid_literal_from_str;

pub fn expand(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as syn::LitStr);
    match guid_literal_from_str(&lit.value()) {
        Ok(ts) => TokenStream::from(ts),
        Err(e) => e.to_compile_error().into(),
    }
}
