#![allow(unused_imports)]
// Test: kernel provider using keyword_mask on event should fail

use fileiolog::etw::etw_provider;

etw_provider! {
    #[etw_provider(kind = "kernel", guid = "00000000-0000-0000-0000-000000000099")]
    pub enum BadEvent {
        #[etw_event(id = 1, version = 0, keyword_mask = 0x10, enable_flag = 0x01)]
        pub struct A {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
