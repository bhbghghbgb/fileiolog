use proc_macro::TokenStream;
use quote::quote;
use std::collections::{BTreeSet, HashMap};
use syn::{
    custom_keyword, Attribute, Error, Field, Ident, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

custom_keyword!(template);

use crate::{
    EtwEventArgs, EtwProviderArgs, EtwProviderKind, guid_literal_from_str, has_skip_in_etw_prop,
    parse_attr_meta,
};

pub fn expand(input: TokenStream) -> TokenStream {
    let provider = parse_macro_input!(input as EtwProviderInput);
    match provider.do_expand() {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

// ── Parsed input ──────────────────────────────────────────────

struct TemplateDef {
    #[allow(dead_code)]
    attrs: Vec<Attribute>,
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

struct EtwProviderInput {
    provider_name: Option<String>,
    provider_guid: Option<String>,
    kind: EtwProviderKind,
    provider_keyword_mask: Option<syn::Expr>,
    provider_enable_flag: Option<syn::Expr>,
    enum_vis: Visibility,
    enum_name: Ident,
    templates: Vec<TemplateDef>,
    variants: Vec<EtwVariant>,
}

struct EtwVariant {
    event_id: u16,
    event_version: Option<u8>,
    event_name: Ident,
    keyword_mask: Option<syn::Expr>,
    enable_flag: Option<syn::Expr>,
    skip: bool,
    attrs: Vec<Attribute>,
    template_name: Ident,
}

impl Parse for EtwProviderInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let outer_attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        let mut provider_name: Option<String> = None;
        let mut provider_guid: Option<String> = None;
        let mut kind = EtwProviderKind::User;
        let mut provider_keyword_mask: Option<syn::Expr> = None;
        let mut provider_enable_flag: Option<syn::Expr> = None;

        for attr in &outer_attrs {
            if attr.path().is_ident("etw_provider") {
                let args: EtwProviderArgs = parse_attr_meta(attr)?;
                provider_name = args.name;
                provider_guid = args.guid;
                kind = args.kind;
                provider_keyword_mask = args.keyword_mask;
                provider_enable_flag = args.enable_flag;

                match kind {
                    EtwProviderKind::User => {
                        if provider_enable_flag.is_some() {
                            return Err(Error::new_spanned(
                                attr,
                                "`enable_flag` on `#[etw_provider(...)]` is only valid for kernel providers (use `keyword_mask` for user providers)",
                            ));
                        }
                    }
                    EtwProviderKind::Kernel => {
                        if provider_keyword_mask.is_some() {
                            return Err(Error::new_spanned(
                                attr,
                                "`keyword_mask` on `#[etw_provider(...)]` is only valid for user providers (use `enable_flag` for kernel providers)",
                            ));
                        }
                    }
                }
            } else {
                return Err(Error::new_spanned(
                    attr,
                    "unsupported attribute; expected `#[etw_provider(...)]`",
                ));
            }
        }

        let has_provider_flag = provider_enable_flag.is_some();

        let enum_vis: Visibility = input.parse()?;
        input.parse::<Token![enum]>()?;
        let enum_name: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut templates: Vec<TemplateDef> = Vec::new();
        let mut variants: Vec<EtwVariant> = Vec::new();

        while !content.is_empty() {
            let attrs: Vec<Attribute> = content.call(Attribute::parse_outer)?;

            if content.peek(template) {
                let _: template = content.parse()?;
                let template_name: Ident = content.parse()?;

                let fields_content;
                syn::braced!(fields_content in content);
                let fields = fields_content.parse_terminated(Field::parse_named, Token![,])?;

                templates.push(TemplateDef {
                    attrs,
                    name: template_name,
                    fields,
                });
            } else {
                let template_name: Ident = content.parse()?;

                let mut event_attr: Option<Attribute> = None;
                let mut other_attrs: Vec<Attribute> = Vec::new();

                for attr in attrs {
                    if attr.path().is_ident("etw_event") {
                        if event_attr.is_some() {
                            return Err(Error::new_spanned(
                                &attr,
                                format!(
                                    "event referencing template `{template_name}` has multiple #[etw_event(...)] attributes; expected exactly one"
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
                        &template_name,
                        format!(
                            "event referencing template `{template_name}` is missing required #[etw_event(...)] attribute"
                        ),
                    )
                })?;

                let args: EtwEventArgs = parse_attr_meta(&event_attr)?;
                let event_name: Ident = Ident::new(&args.name, template_name.span());

                match kind {
                    EtwProviderKind::User => {
                        if args.enable_flag.is_some() {
                            return Err(Error::new_spanned(
                                &event_name,
                                "`enable_flag` is only valid on events in kernel `etw_provider!` \
                                 blocks (use `keyword_mask` for user providers)",
                            ));
                        }
                    }
                    EtwProviderKind::Kernel => {
                        if args.keyword_mask.is_some() {
                            return Err(Error::new_spanned(
                                &event_name,
                                "`keyword_mask` is only valid on events in user `etw_provider!` \
                                 blocks (use `enable_flag` for kernel providers)",
                            ));
                        }
                        if !has_provider_flag && args.enable_flag.is_none() {
                            return Err(Error::new_spanned(
                                &event_name,
                                "`enable_flag` is required on each event when no provider-wide \
                                 `enable_flag` is set on `#[etw_provider(...)]`",
                            ));
                        }
                    }
                }

                variants.push(EtwVariant {
                    event_id: args.id,
                    event_version: args.version,
                    event_name,
                    keyword_mask: args.keyword_mask,
                    enable_flag: args.enable_flag,
                    skip: args.skip,
                    attrs: other_attrs,
                    template_name,
                });
            }

            if !content.is_empty() {
                let _ = content.parse::<Token![,]>();
            }
        }

        Ok(EtwProviderInput {
            provider_name,
            provider_guid,
            kind,
            provider_keyword_mask,
            provider_enable_flag,
            enum_vis,
            enum_name,
            templates,
            variants,
        })
    }
}

impl EtwProviderInput {
    fn do_expand(&self) -> syn::Result<TokenStream> {
        let mut seen_template_names = BTreeSet::new();
        for t in &self.templates {
            if !seen_template_names.insert(&t.name) {
                return Err(Error::new_spanned(
                    &t.name,
                    format!("duplicate template name `{}`", t.name),
                ));
            }
        }

        let templates_map: HashMap<&Ident, &TemplateDef> =
            self.templates.iter().map(|t| (&t.name, t)).collect();

        let mut seen: BTreeSet<(u16, Option<u8>)> = BTreeSet::new();
        for v in &self.variants {
            let key = (v.event_id, v.event_version);
            if !seen.insert(key) {
                let ver_display = match v.event_version {
                    Some(ver) => format!("{}", ver),
                    None => "any (no version)".to_string(),
                };
                return Err(Error::new_spanned(
                    &v.event_name,
                    format!(
                        "duplicate (id, version) pair ({}, {}) in etw_provider! block",
                        v.event_id, ver_display
                    ),
                ));
            }

            if !templates_map.contains_key(&v.template_name) {
                return Err(Error::new_spanned(
                    &v.template_name,
                    format!("template `{}` not found", v.template_name),
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
                let template = templates_map[&v.template_name];
                let attrs = &v.attrs;
                let name = &v.event_name;
                let filtered_fields: Vec<_> = template
                    .fields
                    .iter()
                    .filter(|f| {
                        !f.attrs
                            .iter()
                            .any(|a| a.path().is_ident("etw_prop") && has_skip_in_etw_prop(a))
                    })
                    .collect();
                let template_name_str = v.template_name.to_string();

                let debug_name = format!("{}({})", name, template_name_str);
                let debug_fields: Vec<_> = filtered_fields
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().expect("named fields always have idents");
                        quote! { .field(stringify!(#field_name), &self.#field_name) }
                    })
                    .collect();

                quote! {
                    #(#attrs)*
                    #[derive(Clone, ::fileiolog::etw::EtwEvent)]
                    pub struct #name {
                        #(#filtered_fields),*
                    }

                    impl #name {
                        pub const TEMPLATE_NAME: &'static str = #template_name_str;
                    }

                    impl ::core::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                            f.debug_struct(#debug_name)
                                #(#debug_fields)*
                                .finish()
                        }
                    }
                }
            })
            .collect();

        let enum_variants: Vec<_> = non_skipped
            .iter()
            .map(|v| {
                let name = &v.event_name;
                quote! { #name(#name) }
            })
            .collect();

        let exact_match_arms: Vec<_> = non_skipped
            .iter()
            .filter(|v| v.event_version.is_some())
            .map(|v| {
                let id = v.event_id;
                let ver = v.event_version.unwrap();
                let name = &v.event_name;
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
                let name = &v.event_name;
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
            match self.kind {
                EtwProviderKind::User => {
                    let event_ids: BTreeSet<_> = non_skipped.iter().map(|v| v.event_id).collect();

                    let mut keyword_exprs: Vec<&syn::Expr> = Vec::new();
                    if let Some(ref kw) = self.provider_keyword_mask {
                        keyword_exprs.push(kw);
                    }
                    keyword_exprs
                        .extend(non_skipped.iter().filter_map(|v| v.keyword_mask.as_ref()));

                    let any_method = if !keyword_exprs.is_empty() {
                        let combined = keyword_exprs
                            .iter()
                            .fold(quote! { 0u64 }, |acc, expr| quote! { #acc | #expr });
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
                }
                EtwProviderKind::Kernel => {
                    let mut flag_exprs: Vec<&syn::Expr> = Vec::new();
                    if let Some(ref f) = self.provider_enable_flag {
                        flag_exprs.push(f);
                    }
                    flag_exprs.extend(non_skipped.iter().filter_map(|v| v.enable_flag.as_ref()));

                    let combined_flags = if flag_exprs.is_empty() {
                        quote! { 0u32 }
                    } else {
                        flag_exprs
                            .iter()
                            .fold(quote! { 0u32 }, |acc, expr| quote! { #acc | #expr })
                    };

                    quote! {
                        pub fn build_provider<F>(callback: F) -> ::ferrisetw::provider::Provider
                        where
                            F: Fn(#enum_name) + Send + Sync + 'static,
                        {
                            ::ferrisetw::provider::Provider::kernel(
                                &::ferrisetw::provider::kernel_providers::KernelProvider::new(
                                    PROVIDER_GUID,
                                    #combined_flags,
                                )
                            )
                            .add_callback(move |record: &::ferrisetw::EventRecord, locator: &::ferrisetw::schema_locator::SchemaLocator| {
                                if let Some(event) = #enum_name::try_parse(record, locator) {
                                    callback(event);
                                }
                            })
                            .build()
                        }
                    }
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
                        _ => {
                            #[cfg(debug_assertions)]
                            {
                                log::debug!(
                                    "Unmatched ETW event: event_id={}, version={}, opcode={}, provider=\"{}\", task=\"{}\", opcode_name=\"{}\", decoding_source={:?}",
                                    record.event_id(),
                                    record.version(),
                                    record.opcode(),
                                    schema.provider_name(),
                                    schema.task_name(),
                                    schema.opcode_name(),
                                    schema.decoding_source(),
                                );
                            }
                            None
                        }
                    }
                }
            }

            #build_provider
        };

        Ok(TokenStream::from(expanded))
    }
}
