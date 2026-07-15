#![allow(unused_imports)]
// Test: #[etw_provider] accepts expressions for name/guid

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, guid, EtwEventParse};

pub const PROVIDER_NAME: &str = "TestProvider";
pub const PROVIDER_GUID: ::windows::core::GUID = guid!("DEADBEEF-DEAD-BEEF-DEAD-BEEFDEADBEEF");

etw_provider! {
    #[etw_provider(name = PROVIDER_NAME, guid = PROVIDER_GUID)]
    pub enum TestEvents {
        #[etw_event(id = 1, version = 0)]
        pub struct Alpha {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 2, version = 0)]
        pub struct Beta {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {
    // Constants are user-defined at top of file
    assert_eq!(PROVIDER_NAME, "TestProvider");
    assert_eq!(PROVIDER_GUID, guid!("DEADBEEF-DEAD-BEEF-DEAD-BEEFDEADBEEF"));

    // build_provider compiles and returns a Provider
    let provider = build_provider(|_: TestEvents| {
        // callback – would handle the event
    });
    let _ = provider;
}
