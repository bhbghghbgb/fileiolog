#![allow(unused_imports)]
use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum NoEvent {
        pub struct MissingAttr {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
