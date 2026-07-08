use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Fields, LitStr};

#[proc_macro_derive(EtwEvent, attributes(etw))]
pub fn derive_etw_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic("EtwEvent only supports structs with named fields"),
        },
        _ => panic("EtwEvent only supports structs"),
    };

    let mut prop_values = Vec::new();
    let mut field_names = Vec::new();

    for field in fields {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported");
        let prop = get_etw_prop(&field.attrs).unwrap_or_else(|| {
            panic(format!(
                "Field `{ident}` is missing #[etw(prop = \"...\")]"
            ))
        });
        prop_values.push(prop);
        field_names.push(ident);
    }

    let expanded = quote! {
        impl #name {
            fn try_from_parser(
                parser: &ferrisetw::parser::Parser<'_, '_>,
            ) -> Result<Self, ferrisetw::parser::ParserError> {
                Ok(Self {
                    #( #field_names: parser.try_parse(#prop_values)? ),*
                })
            }
        }
    };

    expanded.into()
}

fn get_etw_prop(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("etw") {
            continue;
        }
        let mut prop = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prop") {
                let value: LitStr = meta.value()?.parse()?;
                prop = Some(value.value());
                Ok(())
            } else {
                Err(meta.error("expected `prop`"))
            }
        });
        return prop;
    }
    None
}

#[cold]
fn panic(msg: impl Into<String>) -> ! {
    panic!("{}", msg.into());
}
