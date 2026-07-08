use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum DupeEvent {
        #[event(id = 10, version = 0)]
        pub struct First {
            #[etw(prop = "X")]
            pub x: u64,
        }

        #[event(id = 10, version = 0)]
        pub struct Second {
            #[etw(prop = "Y")]
            pub y: u64,
        }
    }
}

fn main() {}
