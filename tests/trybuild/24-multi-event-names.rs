#![allow(unused_imports)]
// Test: multiple etw_event attrs with explicit names on a single template struct,
// plus a single-event template that also uses explicit name

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    pub enum MultiNameEvents {
        // Template struct with two events, both with explicit names
        #[etw_event(name = "CreateV0", id = 10, version = 0)]
        #[etw_event(name = "DeleteV0", id = 11, version = 0)]
        pub struct NameCreateArgsV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Single-event template, still uses explicit name
        #[etw_event(name = "CleanupV0", id = 12, version = 0)]
        pub struct CleanupArgsV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
        }
    }
}

fn main() {
    // Verify TEMPLATE_NAME constant is generated
    assert_eq!(CreateV0::TEMPLATE_NAME, "NameCreateArgsV0");
    assert_eq!(DeleteV0::TEMPLATE_NAME, "NameCreateArgsV0");
    assert_eq!(CleanupV0::TEMPLATE_NAME, "CleanupArgsV0");

    // Verify Debug format includes template name
    let create = CreateV0 {
        file_key: 0x1234,
        file_name: String::from("test.txt"),
    };
    let debug_str = format!("{:?}", create);
    assert!(debug_str.starts_with("CreateV0(NameCreateArgsV0)"), "Debug output: {debug_str}");

    // Verify enum variants work
    let _ev = MultiNameEvents::CreateV0(create);
}
