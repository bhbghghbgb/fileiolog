use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::{
    Attribute, Error, Field, Ident, Meta, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

// ───────────────────────────────────────────────────────────────
//  Darling-based attribute argument types
// ───────────────────────────────────────────────────────────────

#[derive(Debug, FromMeta)]
struct EtwPropArgs {
    name: String,
    parse_as: Option<syn::Path>,
    convert_with: Option<syn::Path>,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromMeta)]
struct EtwEventArgs {
    id: u16,
    version: Option<u8>,
    mask: Option<u64>,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, Default, FromMeta)]
struct EtwProviderArgs {
    name: Option<String>,
    guid: Option<String>,
}

/// Minimal struct for checking `skip` without requiring `name`.
#[derive(Default, FromMeta)]
#[darling(default, allow_unknown_fields)]
struct EtwPropSkipCheck {
    #[darling(default)]
    skip: bool,
}

// ───────────────────────────────────────────────────────────────
//  Helpers
// ───────────────────────────────────────────────────────────────

fn parse_attr_meta<T: FromMeta>(attr: &Attribute) -> syn::Result<T> {
    match &attr.meta {
        Meta::List(_) => T::from_meta(&attr.meta).map_err(|e| Error::new_spanned(attr, e)),
        _ => Err(Error::new_spanned(
            attr,
            "expected a list attribute, e.g. `#[name(...)]`",
        )),
    }
}

fn has_skip_in_etw_prop(attr: &Attribute) -> bool {
    match &attr.meta {
        Meta::List(_) => EtwPropSkipCheck::from_meta(&attr.meta)
            .map(|c| c.skip)
            .unwrap_or(false),
        _ => false,
    }
}

// ───────────────────────────────────────────────────────────────
//  Derive macro: #[derive(EtwEvent)]
// ───────────────────────────────────────────────────────────────

#[proc_macro_derive(EtwEvent, attributes(etw_prop))]
pub fn derive_etw_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(ds) => match &ds.fields {
            syn::Fields::Named(named) => &named.named,
            other => {
                return Error::new_spanned(other, "EtwEvent requires a struct with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        syn::Data::Enum(en) => {
            return Error::new_spanned(en.enum_token, "EtwEvent cannot be derived for enums")
                .to_compile_error()
                .into();
        }
        syn::Data::Union(un) => {
            return Error::new_spanned(un.union_token, "EtwEvent cannot be derived for unions")
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
                    Error::new_spanned(
                        f,
                        "missing #[etw_prop(name = \"...\")] attribute on field",
                    )
                })?;

            let content: EtwPropArgs = parse_attr_meta(etw_attr)?;

            if content.convert_with.is_some() && content.parse_as.is_none() {
                return Err(Error::new_spanned(
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

// ───────────────────────────────────────────────────────────────
//  Helper: parse a GUID string into a compile-time literal
// ───────────────────────────────────────────────────────────────

fn guid_literal_from_str(s: &str) -> syn::Result<proc_macro2::TokenStream> {
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
    let lit = parse_macro_input!(input as syn::LitStr);
    match guid_literal_from_str(&lit.value()) {
        Ok(ts) => TokenStream::from(ts),
        Err(e) => e.to_compile_error().into(),
    }
}

// ───────────────────────────────────────────────────────────────
//  Function-like macro: etw_provider! { ... }
// ───────────────────────────────────────────────────────────────

#[proc_macro]
pub fn etw_provider(input: TokenStream) -> TokenStream {
    let provider = parse_macro_input!(input as EtwProviderInput);
    match provider.expand() {
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
    mask: Option<u64>,
    skip: bool,
    attrs: Vec<Attribute>,
    struct_vis: Visibility,
    struct_name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for EtwProviderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Parse optional #[etw_provider(name = "...", guid = "...")]
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

        // Parse: [pub] enum Ident { ... }
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

            // Separate #[etw_event(...)] and other attributes
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

            let args: EtwEventArgs = parse_attr_meta(&event_attr)?;

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

            // Comma separator between enum variants
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
    fn expand(&self) -> syn::Result<TokenStream> {
        // ── Check for duplicate (id, version) pairs across ALL variants ──
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

        // Separate skipped from non-skipped
        let non_skipped: Vec<&EtwVariant> = self.variants.iter().filter(|v| !v.skip).collect();
        let _skipped: Vec<&EtwVariant> = self.variants.iter().filter(|v| v.skip).collect();

        let enum_vis = &self.enum_vis;
        let enum_name = &self.enum_name;

        // ── Generate structs (only non-skipped, filter skipped fields) ──
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
                        !f.attrs.iter().any(|a| {
                            a.path().is_ident("etw_prop") && has_skip_in_etw_prop(a)
                        })
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

        // ── Enum variants (only non-skipped) ────────────────
        let enum_variants: Vec<_> = non_skipped
            .iter()
            .map(|v| {
                let name = &v.struct_name;
                quote! { #name(#name) }
            })
            .collect();

        // ── try_parse match arms ────────────────────────────
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

        // ── Provider constants and build_provider ───────────
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
            let event_ids: Vec<_> = non_skipped.iter().map(|v| v.event_id).collect();

            let all_have_mask = non_skipped.iter().all(|v| v.mask.is_some());
            let combined_mask: u64 = non_skipped
                .iter()
                .filter_map(|v| v.mask)
                .fold(0, |acc, m| acc | m);

            let any_method = if all_have_mask {
                quote! { .any(#combined_mask) }
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
