#![allow(unused_imports)]
// Test: kernel provider with event-level enable_flag builds correctly

use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    #[etw_provider(kind = "kernel", guid = "90cbdc39-4a3e-11d1-84f4-0000f80464e3")]
    pub enum KernelFileEvent {
        #[etw_event(id = 10, version = 0, enable_flag = 0x01000000)]
        pub struct FileIoName {
            #[etw_prop(name = "FileKey", parse_as = ferrisetw::parser::Pointer)]
            pub file_key: usize,
            #[etw_prop(name = "FileName")]
            pub file_name: String,
        }

        #[etw_event(id = 12, version = 0, enable_flag = 0x02000000)]
        pub struct FileIoCreate {
            #[etw_prop(name = "Irp", parse_as = ferrisetw::parser::Pointer)]
            pub irp: usize,
        }
    }
}

fn main() {
    let _provider = build_provider(|_: KernelFileEvent| {});
}
