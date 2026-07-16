#![allow(unused_imports)]
// Test: some structs lack mask → .any() is NOT generated (must still compile)

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    #[etw_provider(name = "PartialMask", guid = "00000000-0000-0000-0000-000000000002")]
    pub enum PartialEvents {
        // Has mask
        #[etw_event(id = 1, version = 0, keyword_mask = 0x10)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        // Lacks keyword_mask → no .any() generated for entire provider
        #[etw_event(id = 2, version = 0)]
        pub struct B {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {
    // Must still compile even though mask is partial
    let _provider = build_provider(|_: PartialEvents| {});
}
