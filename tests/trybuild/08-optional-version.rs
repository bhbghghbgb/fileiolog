#![allow(unused_imports)]
// Test: version is optional; both versioned and versionless variants coexist

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    pub enum VerEvents {
        // Exact version match
        #[etw_event(id = 10, version = 0)]
        pub struct Exact {
            #[etw_prop(name = "F1")]
            pub f1: u64,
        }

        // Versionless – catches all versions of id 10 that don't match Exact
        #[etw_event(id = 10)]
        pub struct AnyVer {
            #[etw_prop(name = "F2")]
            pub f2: u64,
        }

        // Another versionless with different id
        #[etw_event(id = 11)]
        pub struct Other {
            #[etw_prop(name = "F3")]
            pub f3: u64,
        }
    }
}

fn main() {
    let _exact = Exact { f1: 1 };
    let _any   = AnyVer { f2: 2 };
    let _other = Other { f3: 3 };

    let _a = VerEvents::Exact(_exact);
    let _b = VerEvents::AnyVer(_any);
    let _c = VerEvents::Other(_other);
}
