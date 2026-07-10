use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Error, Field, Ident, LitInt, LitStr, Meta, Path, Token, Type, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

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
            let content = parse_etw_field_attr(f)?;
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

/// Parse a single field's `#[etw_prop(name = "...", parse_as = ..., convert_with = ...)]` attribute,
/// returning the parsed content.
fn parse_etw_field_attr(field: &Field) -> syn::Result<EtwFieldAttrContent> {
    let etw_attr = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("etw_prop"))
        .ok_or_else(|| {
            Error::new_spanned(
                field,
                "missing #[etw_prop(name = \"...\")] attribute on field",
            )
        })?;

    let meta = &etw_attr.meta;
    match meta {
        Meta::List(list) => {
            list.parse_args_with(EtwFieldAttrContent::parse)
        }
        _ => Err(Error::new_spanned(
            etw_attr,
            "expected #[etw_prop(name = \"...\")]",
        )),
    }
}

struct EtwFieldAttrContent {
    name: String,
    parse_as: Option<Type>,
    convert_with: Option<Path>,
}

impl Parse for EtwFieldAttrContent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut parse_as: Option<Type> = None;
        let mut convert_with: Option<Path> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key == "name" {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                name = Some(value.value());
            } else if key == "parse_as" {
                input.parse::<Token![=]>()?;
                parse_as = Some(input.parse()?);
            } else if key == "convert_with" {
                input.parse::<Token![=]>()?;
                convert_with = Some(input.parse()?);
            } else {
                return Err(Error::new_spanned(
                    &key,
                    format!(
                        "unknown etw_prop attribute key `{key}`; expected `name`, `parse_as` or `convert_with`"
                    ),
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let name = name.ok_or_else(|| {
            input.error("missing required `name = \"...\"` in etw_prop attribute")
        })?;

        if convert_with.is_some() && parse_as.is_none() {
            return Err(input.error("`convert_with` requires `parse_as` to also be specified"));
        }

        Ok(EtwFieldAttrContent { name, parse_as, convert_with })
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

struct EtwProviderAttr {
    name: Option<String>,
    guid: Option<String>,
}

impl Parse for EtwProviderAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut guid: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;

            if key == "name" {
                name = Some(value.value());
            } else if key == "guid" {
                guid = Some(value.value());
            } else {
                return Err(Error::new_spanned(
                    &key,
                    format!(
                        "unknown etw_provider attribute key `{key}`; expected `name` or `guid`"
                    ),
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(EtwProviderAttr { name, guid })
    }
}

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
                let meta = &attr.meta;
                match meta {
                    Meta::List(list) => {
                        let content = list.parse_args_with(EtwProviderAttr::parse)?;
                        provider_name = content.name;
                        provider_guid = content.guid;
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            "expected #[etw_provider(name = \"...\", guid = \"...\")]",
                        ));
                    }
                }
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

            // Separate #[etw_event(...)], #[etw_skip], and other attributes
            let mut event_attr: Option<Attribute> = None;
            let mut other_attrs: Vec<Attribute> = Vec::new();
            let mut skip = false;

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
                } else if attr.path().is_ident("etw_skip") {
                    skip = true;
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

            let (event_id, event_version, mask) = parse_event_attr(&event_attr)?;

            variants.push(EtwVariant {
                event_id,
                event_version,
                mask,
                skip,
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
        let mut seen: std::collections::BTreeSet<(u16, Option<u8>)> =
            std::collections::BTreeSet::new();
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

        // ── Generate structs (only non-skipped) ──────────────
        let struct_defs: Vec<_> = non_skipped
            .iter()
            .map(|v| {
                let attrs = &v.attrs;
                let vis = &v.struct_vis;
                let name = &v.struct_name;
                let fields = &v.fields;
                quote! {
                    #(#attrs)*
                    #[derive(Debug, Clone, ::fileiolog::etw::EtwEvent)]
                    #vis struct #name {
                        #fields
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
        // Exact (version-specific) arms first, then wildcard (version-agnostic)
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
        let constants = if let Some(guid) = &self.provider_guid {
            let name = self.provider_name.as_deref().unwrap_or("");
            quote! {
                pub const PROVIDER_NAME: &str = #name;
                pub const PROVIDER_GUID: &str = #guid;
            }
        } else {
            quote! {}
        };

        let build_provider = if self.provider_guid.is_some() {
            // Collect event IDs of non-skipped variants
            let event_ids: Vec<_> = non_skipped.iter().map(|v| v.event_id).collect();

            // Check if all non-skipped variants have a mask
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

fn parse_event_attr(attr: &Attribute) -> syn::Result<(u16, Option<u8>, Option<u64>)> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => {
            let ev = list.parse_args_with(EventAttrContent::parse)?;
            Ok((ev.id, ev.version, ev.mask))
        }
        _ => Err(Error::new_spanned(
            attr,
            "expected #[etw_event(id = <int>, version = <int>, mask = <int>)]",
        )),
    }
}

struct EventAttrContent {
    id: u16,
    version: Option<u8>,
    mask: Option<u64>,
}

impl Parse for EventAttrContent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut id: Option<u16> = None;
        let mut version: Option<u8> = None;
        let mut mask: Option<u64> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "id" {
                let lit: LitInt = input.parse()?;
                id = Some(lit.base10_parse::<u16>().map_err(|_| {
                    Error::new_spanned(&lit, "event id must be a u16 integer")
                })?);
            } else if key == "version" {
                let lit: LitInt = input.parse()?;
                version = Some(lit.base10_parse::<u8>().map_err(|_| {
                    Error::new_spanned(&lit, "event version must be a u8 integer")
                })?);
            } else if key == "mask" {
                let lit: LitInt = input.parse()?;
                mask = Some(lit.base10_parse::<u64>().map_err(|_| {
                    Error::new_spanned(&lit, "event mask must be a u64 integer")
                })?);
            } else {
                return Err(Error::new_spanned(
                    &key,
                    format!(
                        "unknown etw_event attribute key `{key}`; expected `id`, `version`, or `mask`"
                    ),
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let id = id.ok_or_else(|| input.error("missing required `id` in etw_event attribute"))?;

        Ok(EventAttrContent { id, version, mask })
    }
}
