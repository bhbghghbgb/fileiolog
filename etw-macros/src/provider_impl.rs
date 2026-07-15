use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::{
    Attribute, Error, Expr, Field, Ident, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

use crate::{
    EtwEventArgs, EtwProviderArgs, guid_literal_from_str, has_skip_in_etw_prop, parse_attr_meta,
};

pub fn expand(input: TokenStream) -> TokenStream {
    let provider = parse_macro_input!(input as EtwProviderInput);
    match provider.do_expand() {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Parsed input ──────────────────────────────────────────────

struct EtwProviderInput {
    provider_name: Option<String>,
    provider_guid: Option<String>,
    enum_vis: Visibility,
    enum_name: Ident,
    variants: Vec<EtwVariant>,
}

struct EtwVariant {
    event_id: u16,
    event_version: Option<u8>,
    mask: Option<Expr>,
    skip: bool,
    attrs: Vec<Attribute>,
    struct_vis: Visibility,
    struct_name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for EtwProviderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let outer_attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        let mut provider_name: Option<String> = None;
        let mut provider_guid: Option<String> = None;

        for attr in &outer_attrs {
            if attr.path().is_ident("etw_provider") {
                let args: EtwProviderArgs = parse_attr_meta(attr)?;
                provider_name = args.name;
                provider_guid = args.guid;
            } else {
                return Err(Error::new_spanned(
                    attr,
                    "unsupported attribute; expected `#[etw_provider(...)]`",
                ));
            }
        }

        let enum_vis: Visibility = input.parse()?;
        input.parse::<Token![enum]>()?;
        let enum_name: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut variants = Vec::new();

        while !content.is_empty() {
            let attrs: Vec<Attribute> = content.call(Attribute::parse_outer)?;
            let struct_vis: Visibility = content.parse()?;
            content.parse::<Token![struct]>()?;
            let struct_name: Ident = content.parse()?;

            let fields_content;
            syn::braced!(fields_content in content);
            let fields = fields_content.parse_terminated(Field::parse_named, Token![,])?;

            let mut event_attr: Option<Attribute> = None;
            let mut other_attrs: Vec<Attribute> = Vec::new();

            for attr in attrs {
                if attr.path().is_ident("etw_event") {
                    if event_attr.is_some() {
                        return Err(Error::new_spanned(
                            &attr,
                            format!(
                                "struct `{struct_name}` has multiple #[etw_event(...)] attributes; expected exactly one"
                            ),
                        ));
                    }
                    event_attr = Some(attr);
                } else {
                    other_attrs.push(attr);
                }
            }

            let event_attr = event_attr.ok_or_else(|| {
                Error::new_spanned(
                    &struct_name,
                    format!(
                        "struct `{struct_name}` is missing required #[etw_event(id = ...)] attribute"
                    ),
                )
            })?;

            let args = EtwEventArgs::from_attr(&event_attr)?;

            variants.push(EtwVariant {
                event_id: args.id,
                event_version: args.version,
                mask: args.mask,
                skip: args.skip,
                attrs: other_attrs,
                struct_vis,
                struct_name,
                fields,
            });

            if !content.is_empty() {
                let _ = content.parse::<Token![,]>();
            }
        }

        Ok(EtwProviderInput {
            provider_name,
            provider_guid,
            enum_vis,
            enum_name,
            variants,
        })
    }
}

impl EtwProviderInput {
    fn do_expand(&self) -> syn::Result<TokenStream> {
        let mut seen: BTreeSet<(u16, Option<u8>)> = BTreeSet::new();
        for v in &self.variants {
            let key = (v.event_id, v.event_version);
            if !seen.insert(key) {
                let ver_display = match v.event_version {
                    Some(ver) => format!("{}", ver),
                    None => "any (no version)".to_string(),
                };
                return Err(Error::new_spanned(
                    &v.struct_name,
                    format!(
                        "duplicate (id, version) pair ({}, {}) in etw_provider! block",
                        v.event_id, ver_display
                    ),
                ));
            }
        }

        let non_skipped: Vec<&EtwVariant> = self.variants.iter().filter(|v| !v.skip).collect();
        let _skipped: Vec<&EtwVariant> = self.variants.iter().filter(|v| v.skip).collect();

        let enum_vis = &self.enum_vis;
        let enum_name = &self.enum_name;

        let struct_defs: Vec<_> = non_skipped
            .iter()
            .map(|v| {
                let attrs = &v.attrs;
                let vis = &v.struct_vis;
                let name = &v.struct_name;
                let filtered_fields: Vec<_> = v
                    .fields
                    .iter()
                    .filter(|f| {
                        !f.attrs
                            .iter()
                            .any(|a| a.path().is_ident("etw_prop") && has_skip_in_etw_prop(a))
                    })
                    .collect();
                quote! {
                    #(#attrs)*
                    #[derive(Debug, Clone, ::fileiolog::etw::EtwEvent)]
                    #vis struct #name {
                        #(#filtered_fields),*
                    }
                }
            })
            .collect();

        let enum_variants: Vec<_> = non_skipped
            .iter()
            .map(|v| {
                let name = &v.struct_name;
                quote! { #name(#name) }
            })
            .collect();

        let exact_match_arms: Vec<_> = non_skipped
            .iter()
            .filter(|v| v.event_version.is_some())
            .map(|v| {
                let id = v.event_id;
                let ver = v.event_version.unwrap();
                let name = &v.struct_name;
                quote! {
                    (#id, #ver) => {
                        Some(Self::#name(
                            ::fileiolog::etw::EtwEventParse::try_from_parser(&parser).ok()?,
                        ))
                    }
                }
            })
            .collect();

        let wildcard_match_arms: Vec<_> = non_skipped
            .iter()
            .filter(|v| v.event_version.is_none())
            .map(|v| {
                let id = v.event_id;
                let name = &v.struct_name;
                quote! {
                    (#id, _) => {
                        Some(Self::#name(
                            ::fileiolog::etw::EtwEventParse::try_from_parser(&parser).ok()?,
                        ))
                    }
                }
            })
            .collect();

        let constants = if let Some(guid_str) = &self.provider_guid {
            let name = self.provider_name.as_deref().unwrap_or("");
            let guid_tokens = guid_literal_from_str(guid_str)?;
            quote! {
                pub const PROVIDER_NAME: &str = #name;
                pub const PROVIDER_GUID: ::windows::core::GUID = #guid_tokens;
            }
        } else {
            quote! {}
        };

        let build_provider = if self.provider_guid.is_some() {
            let event_ids: BTreeSet<_> = non_skipped.iter().map(|v| v.event_id).collect();

            let masks: Vec<&Expr> = non_skipped.iter().filter_map(|v| v.mask.as_ref()).collect();
            let all_have_mask = non_skipped.len() == masks.len() && !masks.is_empty();

            let any_method = if all_have_mask {
                let combined = masks.into_iter().fold(quote! {}, |acc, expr| {
                    if acc.is_empty() {
                        quote! { #expr }
                    } else {
                        quote! { (#acc | #expr) }
                    }
                });
                quote! { .any(#combined) }
            } else {
                quote! {}
            };

            quote! {
                pub fn build_provider<F>(callback: F) -> ::ferrisetw::provider::Provider
                where
                    F: Fn(#enum_name) + Send + Sync + 'static,
                {
                    ::ferrisetw::provider::Provider::by_guid(PROVIDER_GUID)
                        .add_callback(move |record: &::ferrisetw::EventRecord, locator: &::ferrisetw::schema_locator::SchemaLocator| {
                            if let Some(event) = #enum_name::try_parse(record, locator) {
                                callback(event);
                            }
                        })
                        .add_filter(::ferrisetw::provider::EventFilter::ByEventIds(vec![#(#event_ids),*]))
                        #any_method
                        .build()
                }
            }
        } else {
            quote! {}
        };

        let expanded = quote! {
            #constants

            #(#struct_defs)*

            #[derive(Debug, Clone)]
            #enum_vis enum #enum_name {
                #(#enum_variants),*
            }

            impl #enum_name {
                #enum_vis fn try_parse(
                    record: &::ferrisetw::EventRecord,
                    schema_locator: &::ferrisetw::schema_locator::SchemaLocator,
                ) -> Option<Self> {
                    let schema = schema_locator.event_schema(record).ok()?;
                    let parser = ::ferrisetw::parser::Parser::create(record, &schema);
                    match (record.event_id(), record.version()) {
                        #(#exact_match_arms)*
                        #(#wildcard_match_arms)*
                        _ => None,
                    }
                }
            }

            #build_provider
        };

        Ok(TokenStream::from(expanded))
    }
}
