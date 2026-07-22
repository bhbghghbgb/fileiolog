use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Error, Meta};

mod derive_impl;
mod guid_impl;
mod provider_impl;

// ── Proc macro entry points ───────────────────────────────────

#[proc_macro_derive(EtwEvent, attributes(etw_prop))]
pub fn derive_etw_event(input: TokenStream) -> TokenStream {
    derive_impl::expand(input)
}

/// Parses a GUID string literal at compile time into a `windows::core::GUID` value.
///
/// # Example
///
/// ```ignore
/// use etw_macros::guid;
/// const MY_GUID: ::windows::core::GUID = guid!("EDD08927-9CC4-4E65-B970-C2560FB5C289");
/// ```
#[proc_macro]
pub fn guid(input: TokenStream) -> TokenStream {
    guid_impl::expand(input)
}

#[proc_macro]
pub fn etw_provider(input: TokenStream) -> TokenStream {
    provider_impl::expand(input)
}

// ───────────────────────────────────────────────────────────────
//  Darling-based attribute argument types (shared)
// ───────────────────────────────────────────────────────────────

#[derive(Debug, FromMeta)]
pub(crate) struct EtwPropArgs {
    pub(crate) name: String,
    pub(crate) parse_as: Option<syn::Path>,
    pub(crate) convert_with: Option<syn::Path>,
    #[darling(default)]
    pub(crate) skip: bool,
}

#[derive(Debug, FromMeta)]
pub(crate) struct EtwEventArgs {
    pub(crate) id: u16,
    pub(crate) version: Option<u8>,
    pub(crate) name: String,
    pub(crate) keyword_mask: Option<syn::Expr>,
    pub(crate) enable_flag: Option<syn::Expr>,
    #[darling(default)]
    pub(crate) skip: bool,
}

#[derive(Debug, Default, FromMeta)]
#[darling(rename_all = "snake_case")]
pub(crate) enum EtwProviderKind {
    #[default]
    User,
    Kernel,
}

#[derive(Debug, Default, FromMeta)]
pub(crate) struct EtwProviderArgs {
    pub(crate) name: Option<String>,
    pub(crate) guid: Option<String>,
    #[darling(default)]
    pub(crate) kind: EtwProviderKind,
    pub(crate) keyword_mask: Option<syn::Expr>,
    pub(crate) enable_flag: Option<syn::Expr>,
}

/// Minimal struct for checking `skip` without requiring `name`.
#[derive(Default, FromMeta)]
#[darling(default, allow_unknown_fields)]
pub(crate) struct EtwPropSkipCheck {
    #[darling(default)]
    pub(crate) skip: bool,
}

// ───────────────────────────────────────────────────────────────
//  Helpers (shared)
// ───────────────────────────────────────────────────────────────

pub(crate) fn parse_attr_meta<T: FromMeta>(attr: &Attribute) -> syn::Result<T> {
    match &attr.meta {
        Meta::List(_) => T::from_meta(&attr.meta).map_err(|e| Error::new_spanned(attr, e)),
        _ => Err(Error::new_spanned(
            attr,
            "expected a list attribute, e.g. `#[name(...)]`",
        )),
    }
}

pub(crate) fn has_skip_in_etw_prop(attr: &Attribute) -> bool {
    match &attr.meta {
        Meta::List(_) => EtwPropSkipCheck::from_meta(&attr.meta)
            .map(|c| c.skip)
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn guid_literal_from_str(s: &str) -> syn::Result<proc_macro2::TokenStream> {
    if s.len() != 36 {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "GUID string must be exactly 36 characters (format: XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX)",
        ));
    }

    let bytes = s.as_bytes();
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "GUID format must be XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX",
        ));
    }

    let hex_val = |start: usize, len: usize| -> syn::Result<u32> {
        let mut val = 0u32;
        for i in 0..len {
            let b = bytes[start + i];
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                _ => {
                    return Err(Error::new(
                        proc_macro2::Span::call_site(),
                        format!("invalid hex character '{}' in GUID", b as char),
                    ))
                }
            };
            val = (val << 4) | digit;
        }
        Ok(val)
    };

    let data1 = hex_val(0, 8)?;
    let data2 = hex_val(9, 4)? as u16;
    let data3 = hex_val(14, 4)? as u16;
    let data4 = [
        hex_val(19, 2)? as u8,
        hex_val(21, 2)? as u8,
        hex_val(24, 2)? as u8,
        hex_val(26, 2)? as u8,
        hex_val(28, 2)? as u8,
        hex_val(30, 2)? as u8,
        hex_val(32, 2)? as u8,
        hex_val(34, 2)? as u8,
    ];

    Ok(quote! {
        ::windows::core::GUID::from_values(
            #data1,
            #data2,
            #data3,
            [#(#data4),*]
        )
    })
}
