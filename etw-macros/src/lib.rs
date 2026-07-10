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

/// Parse a single field's `#[etw(prop = "...")]` attribute,
/// returning the property name string.
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
            let content = list.parse_args_with(EtwAttrContent::parse)?;
            Ok(content.prop)
        }
        _ => Err(Error::new_spanned(
            etw_attr,
            "expected #[etw(prop = \"...\")]",
        )),
    }
}

struct EtwAttrContent {
    prop: String,
}

impl Parse for EtwAttrContent {
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
        Ok(EtwAttrContent { prop })
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
    enum_vis: Visibility,
    enum_name: Ident,
    variants: Vec<EtwVariant>,
}

struct EtwVariant {
    event_id: u16,
    event_version: u8,
    attrs: Vec<Attribute>,
    struct_vis: Visibility,
    struct_name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for EtwProviderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
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

            let event_attr = event_attr.ok_or_else(|| {
                Error::new_spanned(
                    &struct_name,
                    format!(
                        "struct `{struct_name}` is missing required #[event(id = ..., version = ...)] attribute"
                    ),
                )
            })?;

            let (event_id, event_version) = parse_event_attr(&event_attr)?;

            variants.push(EtwVariant {
                event_id,
                event_version,
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
            enum_vis,
            enum_name,
            variants,
        })
    }
}

impl EtwProviderInput {
    fn expand(&self) -> syn::Result<TokenStream> {
        // ── Check for duplicate (id, version) pairs ──────────
        let mut seen: std::collections::BTreeSet<(u16, u8)> = std::collections::BTreeSet::new();
        for v in &self.variants {
            let key = (v.event_id, v.event_version);
            if !seen.insert(key) {
                return Err(Error::new_spanned(
                    &v.struct_name,
                    format!(
                        "duplicate (id, version) pair ({}, {}) in etw_provider! block",
                        v.event_id, v.event_version
                    ),
                ));
            }
        }

        let enum_vis = &self.enum_vis;
        let enum_name = &self.enum_name;

        // ── Generate each struct ─────────────────────────────
        let struct_defs: Vec<_> = self
            .variants
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
        let enum_variants: Vec<_> = self
            .variants
            .iter()
            .map(|v| {
                let name = &v.struct_name;
                quote! { #name(#name) }
            })
            .collect();

        // ── try_parse match arms ─────────────────────────────
        let match_arms: Vec<_> = self
            .variants
            .iter()
            .map(|v| {
                let id = v.event_id;
                let ver = v.event_version;
                let name = &v.struct_name;
                quote! {
                    (#id, #ver) => {
                        Some(Self::#name(
                            EtwEventParse::try_from_parser(&parser)
                                .ok()?,
                        ))
                    }
                }
            })
            .collect();

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
        };

        Ok(TokenStream::from(expanded))
    }
}

fn parse_event_attr(attr: &Attribute) -> syn::Result<(u16, u8)> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => {
            let ev = list.parse_args_with(EventAttrContent::parse)?;
            Ok((ev.id, ev.version))
        }
        _ => Err(Error::new_spanned(
            attr,
            "expected #[event(id = <int>, version = <int>)]",
        )),
    }
}

struct EventAttrContent {
    id: u16,
    version: u8,
}

impl Parse for EventAttrContent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut id: Option<u16> = None;
        let mut version: Option<u8> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "id" {
                let lit: LitInt = input.parse()?;
                id = Some(
                    lit.base10_parse::<u16>()
                        .map_err(|_| Error::new_spanned(&lit, "event id must be a u16 integer"))?,
                );
            } else if key == "version" {
                let lit: LitInt = input.parse()?;
                version =
                    Some(lit.base10_parse::<u8>().map_err(|_| {
                        Error::new_spanned(&lit, "event version must be a u8 integer")
                    })?);
            } else {
                return Err(Error::new_spanned(
                    &key,
                    format!("unknown event attribute key `{key}`; expected `id` or `version`"),
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let id = id.ok_or_else(|| input.error("missing required `id` in event attribute"))?;
        let version =
            version.ok_or_else(|| input.error("missing required `version` in event attribute"))?;

        Ok(EventAttrContent { id, version })
    }
}
