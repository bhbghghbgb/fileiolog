// Test 1: Happy path — valid struct with #[derive(EtwEvent)]

use ferrisetw::EventRecord;
use ferrisetw::parser::{Parser, ParserError};
use ferrisetw::schema_locator::SchemaLocator;
use fileiolog::etw::{EtwEvent, etw_provider, EtwEventParse};

// ── Test #[derive(EtwEvent)] standalone ──

#[derive(Debug, Clone, EtwEvent)]
pub struct SimpleEvent {
    #[etw_prop(name = "Value")]
    pub value: u64,
    #[etw_prop(name = "Name")]
    pub name: String,
}

// ── Test etw_provider! ──

etw_provider! {
    pub enum MyEvent {
        #[etw_event(id = 1, version = 0)]
        pub struct EventOne {
            #[etw_prop(name = "Id")]
            pub id: u64,
        }

        #[etw_event(id = 2, version = 1)]
        pub struct EventTwo {
            #[etw_prop(name = "Name")]
            pub name: String,
        }
    }
}

fn main() {}
