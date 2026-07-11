#![allow(unused_imports)]
// Test: #[etw_prop(name = "...", skip)] omits the field from the generated struct

use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum MyEvents {
        #[etw_event(id = 1, version = 0)]
        pub struct EventWithSkippedField {
            #[etw_prop(name = "Kept")]
            pub kept: u64,
            #[etw_prop(name = "Skipped", skip)]
            pub skipped: u32,
        }
    }
}

fn main() {
    // The "skipped" field was omitted from the generated struct,
    // so referencing it should fail.
    let _bad = EventWithSkippedField { kept: 42, skipped: 0 };
}
