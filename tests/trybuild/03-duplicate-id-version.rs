use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum DupeEvent {
        #[etw_event(id = 10, version = 0)]
        pub struct First {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 10, version = 0)]
        pub struct Second {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {}
