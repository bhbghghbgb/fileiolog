#![allow(unused_imports)]
// Test: duplicate event struct names should fail

use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum DupeNameEvent {
        #[etw_event(name = "SharedName", id = 1, version = 0)]
        #[etw_event(name = "SharedName", id = 2, version = 0)]
        pub struct TemplateA {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
