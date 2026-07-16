#![allow(unused_imports)]
// Test: user provider with provider-wide enable_flag should fail

use fileiolog::etw::etw_provider;

etw_provider! {
    #[etw_provider(kind = "user", name = "Test", guid = "00000000-0000-0000-0000-000000000099", enable_flag = 0x01)]
    pub enum BadEvent {
        #[etw_event(id = 1, version = 0)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
