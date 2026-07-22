use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeSet;
use syn::{
    Attribute, Error, Field, Ident, Token, Visibility,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

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

struct EtwProviderInput {
    provider_name: Option<String>,
    provider_guid: Option<String>,
    kind: EtwProviderKind,
    provider_keyword_mask: Option<syn::Expr>,
    provider_enable_flag: Option<syn::Expr>,
    enum_vis: Visibility,
    enum_name: Ident,
    variants: Vec<EtwVariant>,
}

struct EtwVariant {
    event_id: u16,
    event_version: Option<u8>,
    keyword_mask: Option<syn::Expr>,
    enable_flag: Option<syn::Expr>,
    skip: bool,
    attrs: Vec<Attribute>,
    struct_vis: Visibility,
    struct_name: Ident,
    template_name: String,
    fields: Punctuated<Field, Token![,]>,
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

        let mut variants = Vec::new();

        while !content.is_empty() {
            let attrs: Vec<Attribute> = content.call(Attribute::parse_outer)?;
            let struct_vis: Visibility = content.parse()?;
            content.parse::<Token![struct]>()?;
            let template_struct_name: Ident = content.parse()?;

            let fields_content;
            syn::braced!(fields_content in content);
            let fields = fields_content.parse_terminated(Field::parse_named, Token![,])?;

            let mut event_attrs: Vec<Attribute> = Vec::new();
            let mut other_attrs: Vec<Attribute> = Vec::new();

            for attr in attrs {
                if attr.path().is_ident("etw_event") {
                    event_attrs.push(attr);
                } else {
                    other_attrs.push(attr);
                }
            }

            if event_attrs.is_empty() {
                return Err(Error::new_spanned(
                    &template_struct_name,
                    format!(
                        "struct `{template_struct_name}` is missing required #[etw_event(id = ...)] attribute"
                    ),
                ));
            }

            let mut event_args_list: Vec<EtwEventArgs> = Vec::new();
            for attr in &event_attrs {
                let args: EtwEventArgs = parse_attr_meta(attr)?;
                event_args_list.push(args);
            }

            if event_args_list.len() > 1 {
                for args in &event_args_list {
                    if args.name.is_none() {
                        return Err(Error::new_spanned(
                            &template_struct_name,
                            format!(
                                "struct `{template_struct_name}` has multiple #[etw_event] attributes \
                                 without an explicit `name` on each; when using multiple #[etw_event] \
                                 attributes on a single template struct, all must specify `name = \"...\"` \
                                 to avoid duplicate struct names"
                            ),
                        ));
                    }
                }
            }

            for args in event_args_list {
                let resolved_name = if let Some(ref name) = args.name {
                    Ident::new(name, template_struct_name.span())
                } else {
                    template_struct_name.clone()
                };

                match kind {
                    EtwProviderKind::User => {
                        if args.enable_flag.is_some() {
                            return Err(Error::new_spanned(
                                &resolved_name,
                                "`enable_flag` is only valid on events in kernel `etw_provider!` \
                                 blocks (use `keyword_mask` for user providers)",
                            ));
                        }
                    }
                    EtwProviderKind::Kernel => {
                        if args.keyword_mask.is_some() {
                            return Err(Error::new_spanned(
                                &resolved_name,
                                "`keyword_mask` is only valid on events in user `etw_provider!` \
                                 blocks (use `enable_flag` for kernel providers)",
                            ));
                        }
                        if !has_provider_flag && args.enable_flag.is_none() {
                            return Err(Error::new_spanned(
                                &resolved_name,
                                "`enable_flag` is required on each event when no provider-wide \
                                 `enable_flag` is set on `#[etw_provider(...)]`",
                            ));
                        }
                    }
                }

                variants.push(EtwVariant {
                    event_id: args.id,
                    event_version: args.version,
                    keyword_mask: args.keyword_mask,
                    enable_flag: args.enable_flag,
                    skip: args.skip,
                    attrs: other_attrs.clone(),
                    struct_vis: struct_vis.clone(),
                    struct_name: resolved_name,
                    template_name: template_struct_name.to_string(),
                    fields: fields.clone(),
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

        let mut seen_names: BTreeSet<String> = BTreeSet::new();
        for v in &self.variants {
            let name = v.struct_name.to_string();
            if !seen_names.insert(name.clone()) {
                return Err(Error::new_spanned(
                    &v.struct_name,
                    format!(
                        "duplicate event struct name `{name}` in etw_provider! block"
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
                let template = &v.template_name;
                let filtered_fields: Vec<_> = v
                    .fields
                    .iter()
                    .filter(|f| {
                        !f.attrs
                            .iter()
                            .any(|a| a.path().is_ident("etw_prop") && has_skip_in_etw_prop(a))
                    })
                    .collect();

                let field_debug_stmts: Vec<_> = filtered_fields
                    .iter()
                    .filter_map(|f| {
                        f.ident.as_ref().map(|ident| {
                            quote! { .field(stringify!(#ident), &self.#ident) }
                        })
                    })
                    .collect();

                quote! {
                    #(#attrs)*
                    #[derive(Clone, ::fileiolog::etw::EtwEvent)]
                    #vis struct #name {
                        #(#filtered_fields),*
                    }

                    impl #name {
                        pub const TEMPLATE_NAME: &str = #template;
                    }

                    impl ::std::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                            f.debug_struct(concat!(stringify!(#name), "(", #template, ")"))
                                #(#field_debug_stmts)*
                                .finish()
                        }
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
