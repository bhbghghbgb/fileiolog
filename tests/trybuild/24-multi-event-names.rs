#![allow(unused_imports)]
// Test: multiple etw_event attrs with explicit names on a single template struct

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    pub enum MultiNameEvents {
        // Template struct with two events, both with explicit names
        #[etw_event(name = "CreateV0", id = 10, version = 0)]
        #[etw_event(name = "DeleteV0", id = 11, version = 0)]
        pub struct FileIoNameV0 {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        // Separate template struct, single event (name defaults to struct name)
        #[etw_event(id = 12, version = 0)]
        pub struct CleanupV0 {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
        }
    }
}

fn main() {
    // Verify TEMPLATE_NAME constant is generated
    assert_eq!(CreateV0::TEMPLATE_NAME, "FileIoNameV0");
    assert_eq!(DeleteV0::TEMPLATE_NAME, "FileIoNameV0");
    assert_eq!(CleanupV0::TEMPLATE_NAME, "CleanupV0");

    // Verify Debug format includes template name
    let create = CreateV0 {
        file_key: 0x1234,
        file_name: String::from("test.txt"),
    };
    let debug_str = format!("{:?}", create);
    assert!(debug_str.starts_with("CreateV0(FileIoNameV0)"), "Debug output: {debug_str}");

    // Verify enum variants work
    let _ev = MultiNameEvents::CreateV0(create);
}
