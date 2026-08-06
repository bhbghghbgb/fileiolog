use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced, parse::Parse, parse::ParseStream, Attribute, Error, Ident, Token, Visibility,
};

/// Input: the entire `fileio_events! { ... }` invocation
struct FileIoEventsInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    events: Vec<FileIoEvent>,
}

/// A single event definition: `Name = (opcode, version) { fields... }`
struct FileIoEvent {
    name: Ident,
    opcode: u8,
    version: u8,
    fields: Vec<FileIoField>,
}

/// A single field: `FieldName: type_keyword "prop_name"`
struct FileIoField {
    name: Ident,
    ty: syn::Type,
    prop_name: String,
}

impl Parse for FileIoEventsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![enum]>()?;
        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);
        let events = content.parse_terminated(FileIoEvent::parse, Token![,])?;

        Ok(FileIoEventsInput {
            attrs,
            vis,
            name,
            events: events.into_iter().collect(),
        })
    }
}

impl Parse for FileIoEvent {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        // Parse (opcode, version)
        let tuple_content;
        syn::parenthesized!(tuple_content in input);
        let opcode: syn::LitInt = tuple_content.parse()?;
        tuple_content.parse::<Token![,]>()?;
        let version: syn::LitInt = tuple_content.parse()?;

        let opcode_val: u8 = opcode.base10_parse()?;
        let version_val: u8 = version.base10_parse()?;

        // Parse fields block
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            let field_name: Ident = fields_content.parse()?;
            fields_content.parse::<Token![:]>()?;

            // Parse type keyword
            let type_ident: Ident = fields_content.parse()?;
            let ty = match type_ident.to_string().as_str() {
                "pointer" => syn::parse_str::<syn::Type>("usize").unwrap(),
                "string" => syn::parse_str::<syn::Type>("String").unwrap(),
                "u32" => syn::parse_str::<syn::Type>("u32").unwrap(),
                "u64" => syn::parse_str::<syn::Type>("u64").unwrap(),
                "i32" => syn::parse_str::<syn::Type>("i32").unwrap(),
                "i64" => syn::parse_str::<syn::Type>("i64").unwrap(),
                "filetime" => syn::parse_str::<syn::Type>("u64").unwrap(),
                _ => {
                    return Err(Error::new(
                        type_ident.span(),
                        format!(
                            "unknown field type `{}`; expected one of: pointer, string, u32, u64, i32, i64, filetime",
                            type_ident
                        ),
                    ))
                }
            };

            // Parse property name string
            let prop_name: syn::LitStr = fields_content.parse()?;

            fields.push(FileIoField {
                name: field_name,
                ty,
                prop_name: prop_name.value(),
            });

            if !fields_content.is_empty() {
                fields_content.parse::<Token![,]>()?;
            }
        }

        Ok(FileIoEvent {
            name,
            opcode: opcode_val,
            version: version_val,
            fields,
        })
    }
}

impl FileIoEventsInput {
    fn expand(&self) -> proc_macro2::TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let enum_name = &self.name;

        // Generate struct definitions with derive(EtwEvent) and etw_prop attributes
        let struct_defs: Vec<_> = self
            .events
            .iter()
            .map(|event| {
                let name = &event.name;
                let opcode = event.opcode;
                let version = event.version;

                let field_idents: Vec<&Ident> = event.fields.iter().map(|f| &f.name).collect();

                let fields = event.fields.iter().map(|f| {
                    let field_name = &f.name;
                    let ty = &f.ty;
                    let prop_name = &f.prop_name;
                    let needs_parse_as = matches!(f.ty.to_token_stream().to_string().as_str(), "usize");
                    if needs_parse_as {
                        quote! {
                            #[etw_prop(name = #prop_name, parse_as = ::ferrisetw::parser::Pointer)]
                            pub #field_name: #ty,
                        }
                    } else {
                        quote! {
                            #[etw_prop(name = #prop_name)]
                            pub #field_name: #ty,
                        }
                    }
                });

                let field_debug_stmts: Vec<_> = field_idents
                    .iter()
                    .map(|ident| {
                        quote! { .field(stringify!(#ident), &self.#ident) }
                    })
                    .collect();

                quote! {
                    #[derive(Clone, ::fileiolog::etw::EtwEvent)]
                    #[allow(non_snake_case)]
                    pub struct #name {
                        #(#fields)*
                    }

                    impl #name {
                        pub const OPCODE: u8 = #opcode;
                        pub const VERSION: u8 = #version;
                    }

                    impl ::std::fmt::Debug for #name {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                            f.debug_struct(stringify!(#name))
                                #(#field_debug_stmts)*
                                .finish()
                        }
                    }
                }
            })
            .collect();

        // Generate enum variants
        let enum_variants: Vec<_> = self
            .events
            .iter()
            .map(|event| {
                let name = &event.name;
                quote! { #name(#name) }
            })
            .collect();

        // Generate match arms for try_parse (exact version match first)
        let exact_match_arms: Vec<_> = self
            .events
            .iter()
            .map(|event| {
                let opcode: u16 = event.opcode as u16;
                let version = event.version;
                let name = &event.name;
                quote! {
                    (#opcode, #version) => {
                        Some(Self::#name(
                            ::fileiolog::etw::EtwEventParse::try_from_parser(&parser).ok()?,
                        ))
                    }
                }
            })
            .collect();

        let expanded = quote! {
            #(#struct_defs)*

            #[derive(Debug, Clone)]
            #[allow(dead_code)]
            #(#attrs)*
            #vis enum #enum_name {
                #(#enum_variants),*
            }

            impl #enum_name {
                #vis fn try_parse(
                    record: &::ferrisetw::EventRecord,
                    schema_locator: &::ferrisetw::schema_locator::SchemaLocator,
                ) -> Option<Self> {
                    let schema = schema_locator.event_schema(record).ok()?;
                    let parser = ::ferrisetw::parser::Parser::create(record, &schema);
                    match (record.event_id(), record.version()) {
                        #(#exact_match_arms)*
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
        };

        expanded
    }
}

#[proc_macro]
pub fn fileio_events(input: TokenStream) -> TokenStream {
    let parsed: FileIoEventsInput = syn::parse_macro_input!(input);
    let expanded = parsed.expand();
    TokenStream::from(expanded)
}
