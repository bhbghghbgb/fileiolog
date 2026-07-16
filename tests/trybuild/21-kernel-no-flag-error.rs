#![allow(unused_imports)]
// Test: kernel provider with no enable_flag anywhere should fail

use fileiolog::etw::etw_provider;

etw_provider! {
    #[etw_provider(kind = "kernel", guid = "00000000-0000-0000-0000-000000000099")]
    pub enum BadEvent {
        #[etw_event(id = 1, version = 0)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
