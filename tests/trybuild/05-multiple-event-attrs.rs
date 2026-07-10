use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum MultiEvent {
        #[etw_event(id = 1, version = 0)]
        #[etw_event(id = 2, version = 0)]
        pub struct TwoEvents {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {}
