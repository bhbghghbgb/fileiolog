// Test 13: #[etw_prop] with parse_as and convert_with

use ferrisetw::parser::Pointer;
use fileiolog::etw::{EtwEvent, EtwPropConvert, etw_provider, EtwEventParse};

// ── Test trait-based conversion (parse_as only) ──

#[derive(Debug, Clone, EtwEvent)]
pub struct WithParseAs {
    #[etw_prop(name = "FileKey", parse_as = Pointer)]
    pub file_key: usize,
    #[etw_prop(name = "FileName")]
    pub file_name: String,
}

// ── Test custom conversion (parse_as + convert_with) ──

fn double_it(val: u64) -> u64 {
    val * 2
}

#[derive(Debug, Clone, EtwEvent)]
pub struct WithConvertFn {
    #[etw_prop(name = "Value", parse_as = u64, convert_with = double_it)]
    pub doubled: u64,
}

// ── Test inside etw_provider! ──

etw_provider! {
    pub enum TestEvent {
        #[etw_event(id = 1, version = 0)]
        pub struct EventOne {
            #[etw_prop(name = "FileKey", parse_as = Pointer)]
            pub file_key: usize,
        }

        #[etw_event(id = 2)]
        pub struct EventTwo {
            #[etw_prop(name = "Name")]
            pub name: String,
            #[etw_prop(name = "Value", parse_as = u64, convert_with = double_it)]
            pub doubled: u64,
        }
    }
}

fn main() {}
