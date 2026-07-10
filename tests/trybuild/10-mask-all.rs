// Test: all structs have mask → .any(COMBINED) is generated in build_provider

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    #[etw_provider(name = "MaskedTest", guid = "00000000-0000-0000-0000-000000000001")]
    pub enum MaskedEvents {
        #[etw_event(id = 1, version = 0, mask = 0x10)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 2, version = 0, mask = 0x20)]
        pub struct B {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }

        #[etw_event(id = 3, version = 0, mask = 0x40)]
        pub struct C {
            #[etw_prop(name = "Z")]
            pub z: u64,
        }
    }
}

fn main() {
    // build_provider must compile (including .any())
    let _provider = build_provider(|_: MaskedEvents| {});
}
