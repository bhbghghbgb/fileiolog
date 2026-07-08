use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum NoEvent {
        pub struct MissingAttr {
            #[etw(prop = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
