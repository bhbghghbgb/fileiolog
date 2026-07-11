use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{parse_attr_meta, EtwPropArgs};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => &named.named,
            other => {
                return syn::Error::new_spanned(
                    other,
                    "EtwEvent requires a struct with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        syn::Data::Enum(en) => {
            return syn::Error::new_spanned(en.enum_token, "EtwEvent cannot be derived for enums")
                .to_compile_error()
                .into();
        }
        syn::Data::Union(un) => {
            return syn::Error::new_spanned(
                un.union_token,
                "EtwEvent cannot be derived for unions",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_parses: Vec<_> = match fields
        .iter()
        .map(|f| {
            let field_name = f
                .ident
                .as_ref()
                .expect("named struct fields always have idents");

            let etw_attr = f
                .attrs
                .iter()
                .find(|a| a.path().is_ident("etw_prop"))
                .ok_or_else(|| {
                    syn::Error::new_spanned(
                        f,
                        "missing #[etw_prop(name = \"...\")] attribute on field",
                    )
                })?;

            let content: EtwPropArgs = parse_attr_meta(etw_attr)?;

            if content.convert_with.is_some() && content.parse_as.is_none() {
                return Err(syn::Error::new_spanned(
                    etw_attr,
                    "`convert_with` requires `parse_as` to also be specified",
                ));
            }

            if content.skip {
                return Ok(quote! { #field_name: Default::default() });
            }

            let name = &content.name;
            let parse = if let Some(parse_as) = &content.parse_as {
                let field_type = &f.ty;
                let conversion = if let Some(convert_with) = &content.convert_with {
                    quote! { #convert_with(__val) }
                } else {
                    quote! { <#field_type as ::fileiolog::etw::EtwPropConvert<#parse_as>>::convert(__val) }
                };
                quote! {{
                    let __val: #parse_as = parser.try_parse(#name)?;
                    #conversion
                }}
            } else {
                quote! { parser.try_parse(#name)? }
            };
            Ok(quote! { #field_name: #parse })
        })
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        impl ::fileiolog::etw::EtwEventParse for #struct_name {
            fn try_from_parser(
                parser: &::ferrisetw::parser::Parser<'_, '_>,
            ) -> Result<Self, ::ferrisetw::parser::ParserError> {
                Ok(Self {
                    #(#field_parses),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
