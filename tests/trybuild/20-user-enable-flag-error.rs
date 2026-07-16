#![allow(unused_imports)]
// Test: user provider using enable_flag on event should fail

use fileiolog::etw::etw_provider;

etw_provider! {
    #[etw_provider(kind = "user")]
    pub enum BadEvent {
        #[etw_event(id = 1, version = 0, enable_flag = 0x01)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
