#![allow(unused_imports)]
use fileiolog::etw::etw_provider;

// Test: template syntax — multiple #[etw_event(name = "...")] on one struct
etw_provider! {
    pub enum MultiEvent {
        #[etw_event(name = "FirstEvent", id = 1, version = 0)]
        #[etw_event(name = "SecondEvent", id = 2, version = 0)]
        pub struct TemplateStruct {
            #[etw_prop(name = "X")]
            pub x: u64,
        }
    }
}

fn main() {
    // Verify the generated structs exist and have TEMPLATE_NAME
    let _first = FirstEvent { x: 42 };
    let _second = SecondEvent { x: 99 };
    assert_eq!(FirstEvent::TEMPLATE_NAME, "TemplateStruct");
    assert_eq!(SecondEvent::TEMPLATE_NAME, "TemplateStruct");
}
