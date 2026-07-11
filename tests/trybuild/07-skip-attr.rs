// Test: #[etw_event(skip)] excludes structs from codegen

use fileiolog::etw::etw_provider;

etw_provider! {
    pub enum SkippedEvents {
        #[etw_event(id = 1, version = 0)]
        pub struct Included {
            #[etw_prop(name = "X")]
            pub x: u64,
        }

        #[etw_event(id = 2, version = 0, skip)]
        pub struct ZzRemovedEvent {
            #[etw_prop(name = "Y")]
            pub y: u64,
        }
    }
}

fn main() {
    // Included struct can be used
    let e = Included { x: 42 };
    let _ev = SkippedEvents::Included(e);

    // ZzRemovedEvent struct does NOT exist in codegen — referencing it should fail
    let _bad = ZzRemovedEvent { y: 0 };

    match _ev {
        SkippedEvents::Included(_) => {}
    }
}
