#![allow(unused_imports)]
// Test: user provider with provider-wide keyword_mask

use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    #[etw_provider(kind = "user", name = "Test", guid = "00000000-0000-0000-0000-000000000003", keyword_mask = 0xFF)]
    pub enum TestEvents {
        #[etw_event(id = 1, version = 0)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 2, version = 0)]
        pub struct B {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {
    let _provider = build_provider(|_: TestEvents| {});
}
