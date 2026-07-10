// Test: #[etw_provider] generates constants and build_provider

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::provider::{EventFilter, Provider};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    #[etw_provider(name = "TestProvider", guid = "DEADBEEF-DEAD-BEEF-DEAD-BEEFDEADBEEF")]
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
    // Verify constants are generated
    assert_eq!(PROVIDER_NAME, "TestProvider");
    assert_eq!(PROVIDER_GUID, "DEADBEEF-DEAD-BEEF-DEAD-BEEFDEADBEEF");

    // Verify build_provider compiles and returns a Provider
    let provider = build_provider(|_: TestEvents| {
        // callback – would handle the event
    });
    let _ = provider;
}
