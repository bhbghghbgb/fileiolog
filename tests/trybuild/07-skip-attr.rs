// Test: #[etw_skip] excludes structs from codegen

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

etw_provider! {
    pub enum SkippedEvents {
        #[etw_event(id = 1, version = 0)]
        pub struct Included {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 2, version = 0)]
        #[etw_skip]
        pub struct Excluded {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {
    // Included struct can be used
    let e = Included { x: 42 };
    let _ev = SkippedEvents::Included(e);

    // Excluded struct does NOT exist in codegen.
    // Uncommenting the next line would fail to compile:
    // let _bad = SkippedEvents::Excluded(Excluded { y: 0 });

    // The enum only has Included variant
    match _ev {
        SkippedEvents::Included(_) => {}
    }
}
