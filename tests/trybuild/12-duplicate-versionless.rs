#![allow(unused_imports)]
use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum DupVerless {
        #[etw_event(id = 10)]
        pub struct First {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        // Same id, no version → duplicate
        #[etw_event(id = 10)]
        pub struct Second {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {}
