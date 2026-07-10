use std::collections::BTreeSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Error, Field, Ident, LitInt, LitStr, Meta, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

// ───────────────────────────────────────────────────────────────
//  Derive macro: #[derive(EtwEvent)]
// ───────────────────────────────────────────────────────────────

#[proc_macro_derive(EtwEvent, attributes(etw))]
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
            let prop = parse_etw_field_attr(f)?;
            Ok(quote! { #field_name: parser.try_parse(#prop)? })
        })
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        impl EtwEventParse for #struct_name {
            fn try_from_parser(
                parser: &Parser<'_, '_>,
            ) -> Result<Self, ParserError> {
                Ok(Self {
                    #(#field_parses),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_etw_field_attr(field: &Field) -> syn::Result<String> {
    let etw_attr = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("etw"))
        .ok_or_else(|| {
            Error::new_spanned(field, "missing #[etw(prop = \"...\")] attribute on field")
        })?;

    let meta = &etw_attr.meta;
    match meta {
        Meta::List(list) => {
            let content = list.parse_args_with(EtwFieldAttrContent::parse)?;
            Ok(content.prop)
        }
        _ => Err(Error::new_spanned(
            etw_attr,
            "expected #[etw(prop = \"...\")]",
        )),
    }
}

struct EtwFieldAttrContent {
    prop: String,
}

impl Parse for EtwFieldAttrContent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut prop: Option<String> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key == "prop" {
                input.parse::<Token![=]>()?;
                let value: LitStr = input.parse()?;
                prop = Some(value.value());
            } else {
                return Err(Error::new_spanned(
                    &key,
                    format!("unknown etw attribute key `{key}`; expected `prop`"),
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let prop =
            prop.ok_or_else(|| input.error("missing required `prop = \"...\"` in etw attribute"))?;
        Ok(EtwFieldAttrContent { prop })
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
    keyword_mask: Option<u64>,
    enum_vis: Visibility,
    enum_name: Ident,
    variants: Vec<EtwVariant>,
}

struct EtwVariant {
    event_id: Option<u16>,
    event_version: Option<u8>,
    skip: bool,
    attrs: Vec<Attribute>,
    struct_vis: Visibility,
    struct_name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for EtwProviderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        // Parse optional "keywords = 0x..." prefix before the enum
        let keyword_mask = if input.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;
            if ident == "keywords" {
                input.parse::<Token![=]>()?;
                let lit: LitInt = input.parse()?;
                let mask = lit.base10_parse::<u64>().map_err(|_| {
                    Error::new_spanned(&lit, "keyword mask must be a u64 integer")
                })?;
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
                Some(mask)
            } else {
                return Err(Error::new_spanned(
                    &ident,
                    "expected `keywords = <mask>` or `pub`",
                ));
            }
        } else {
            None
        };

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

            // Separate #[event(...)] from other attributes
            let mut event_attr: Option<Attribute> = None;
            let mut other_attrs: Vec<Attribute> = Vec::new();

            for attr in attrs {
                if attr.path().is_ident("event") {
                    if event_attr.is_some() {
                        return Err(Error::new_spanned(
                            &attr,
                            format!(
                                "struct `{struct_name}` has multiple #[event(...)] attributes; expected exactly one"
                            ),
                        ));
                    }
                    event_attr = Some(attr);
                } else {
                    other_attrs.push(attr);
                }
            }

            let Some(event_attr) = event_attr else {
                return Err(Error::new_spanned(
                    &struct_name,
                    format!(
                        "struct `{struct_name}` is missing required #[event(...)] attribute"
                    ),
                ));
            };

            let ev_meta = parse_event_attr(&event_attr)?;

            variants.push(EtwVariant {
                event_id: ev_meta.id,
                event_version: ev_meta.version,
                skip: ev_meta.skip,
                attrs: other_attrs,
                struct_vis,
                struct_name,
                fields,
            });

            // Optional comma separator between enum variants
            if !content.is_empty() {
                let _ = content.parse::<Token![,]>();
            }
        }

        Ok(EtwProviderInput {
            keyword_mask,
            enum_vis,
            enum_name,
            variants,
        })
    }
}

// ── Event attribute content ───────────────────────────────────

struct EventAttrContent {
    id: Option<u16>,
    version: Option<u8>,
    skip: bool,
}

impl Parse for EventAttrContent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut id: Option<u16> = None;
        let mut version: Option<u8> = None;
        let mut skip = false;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            if key == "skip" {
                skip = true;
            } else {
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
                } else {
                    return Err(Error::new_spanned(
                        &key,
                        format!(
                            "unknown event attribute key `{key}`; expected `id`, `version`, or `skip`"
                        ),
                    ));
                }
            }

            if !input.is_empty() && input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if !skip && id.is_none() {
            return Err(input.error("missing required `id` in event attribute"));
        }

        Ok(EventAttrContent { id, version, skip })
    }
}

fn parse_event_attr(attr: &Attribute) -> syn::Result<EventAttrContent> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => list.parse_args_with(EventAttrContent::parse),
        _ => Err(Error::new_spanned(
            attr,
            "expected #[event(id = <int>, version = <int>)] or #[event(skip)]",
        )),
    }
}

// ── Code generation ───────────────────────────────────────────

impl EtwProviderInput {
    fn expand(&self) -> syn::Result<TokenStream> {
        // ── Check for duplicate (id, version) pairs ──────────
        let mut seen: BTreeSet<(Option<u16>, Option<u8>)> = BTreeSet::new();
        for v in &self.variants {
            if v.skip {
                continue;
            }
            let key = (v.event_id, v.event_version);
            if !seen.insert(key) {
                let id_display = match v.event_id {
                    Some(id) => id.to_string(),
                    None => "?".to_string(),
                };
                let ver_display = match v.event_version {
                    Some(ver) => format!("{ver}"),
                    None => "any".to_string(),
                };
                return Err(Error::new_spanned(
                    &v.struct_name,
                    format!(
                        "duplicate event spec (id = {id_display}, version = {ver_display}) \
                         in etw_provider! block"
                    ),
                ));
            }
        }

        let enum_vis = &self.enum_vis;
        let enum_name = &self.enum_name;

        let active_variants: Vec<&EtwVariant> =
            self.variants.iter().filter(|v| !v.skip).collect();

        // ── Generate each struct ─────────────────────────────
        let struct_defs: Vec<_> = active_variants
            .iter()
            .map(|v| {
                let attrs = &v.attrs;
                let vis = &v.struct_vis;
                let name = &v.struct_name;
                let fields = &v.fields;
                quote! {
                    #(#attrs)*
                    #[derive(Debug, Clone, EtwEvent)]
                    #vis struct #name {
                        #fields
                    }
                }
            })
            .collect();

        // ── Enum variants ────────────────────────────────────
        let enum_variants: Vec<_> = active_variants
            .iter()
            .map(|v| {
                let name = &v.struct_name;
                quote! { #name(#name) }
            })
            .collect();

        // ── try_parse match arms ─────────────────────────────
        // Order: versioned arms first, then version-less arms (for the same event_id).
        // This way versioned ones take precedence when both exist.
        let mut id_groups: BTreeMap<u16, Vec<&EtwVariant>> = BTreeMap::new();
        for v in &active_variants {
            if let Some(id) = v.event_id {
                id_groups.entry(id).or_default().push(v);
            }
        }
        let match_arms: Vec<_> = id_groups
            .values()
            .flat_map(|variants| {
                let mut sorted = variants.clone();
                // versioned (Some) before version-less (None)
                sorted.sort_by_key(|v| v.event_version.is_none());
                sorted.into_iter().map(|v| {
                    let id = v.event_id.unwrap();
                    let name = &v.struct_name;
                    match v.event_version {
                        Some(ver) => {
                            quote! {
                                (#id, #ver) => {
                                    Some(Self::#name(
                                        EtwEventParse::try_from_parser(&parser).ok()?,
                                    ))
                                }
                            }
                        }
                        None => {
                            quote! {
                                (#id, _) => {
                                    Some(Self::#name(
                                        EtwEventParse::try_from_parser(&parser).ok()?,
                                    ))
                                }
                            }
                        }
                    }
                })
            })
            .collect();

        use std::collections::BTreeMap;

        // ── Event IDs for ByEventIds filter ─────────────────
        let mut unique_ids: Vec<u16> = active_variants
            .iter()
            .filter_map(|v| v.event_id)
            .collect();
        unique_ids.sort();
        unique_ids.dedup();

        // ── build_provider function ─────────────────────────
        let any_method = self
            .keyword_mask
            .map(|mask| quote! { .any(#mask) })
            .unwrap_or_else(|| quote! {});

        let build_provider = quote! {
            pub fn build_provider<F>(callback: F) -> Provider
            where
                F: Fn(#enum_name) + Send + Sync + 'static,
            {
                Provider::by_guid(PROVIDER_GUID)
                    .add_callback(move |record, locator| {
                        if let Some(event) = #enum_name::try_parse(record, locator) {
                            callback(event);
                        }
                    })
                    #any_method
                    .add_filter(EventFilter::ByEventIds(
                        vec![#(#unique_ids),*],
                    ))
                    .build()
            }
        };

        let expanded = quote! {
            #(#struct_defs)*

            #[derive(Debug, Clone)]
            #enum_vis enum #enum_name {
                #(#enum_variants),*
            }

            impl #enum_name {
                #enum_vis fn try_parse(
                    record: &EventRecord,
                    schema_locator: &SchemaLocator,
                ) -> Option<Self> {
                    let schema = schema_locator
                        .event_schema(record)
                        .ok()?;
                    let parser = Parser::create(record, &schema);
                    match (record.event_id(), record.version()) {
                        #(#match_arms)*
                        _ => None,
                    }
                }
            }

            #build_provider
        };

        Ok(TokenStream::from(expanded))
    }
}
