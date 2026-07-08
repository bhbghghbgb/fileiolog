use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum MultiEvent {
        #[event(id = 1, version = 0)]
        #[event(id = 2, version = 0)]
        pub struct TwoEvents {
            #[etw(prop = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
