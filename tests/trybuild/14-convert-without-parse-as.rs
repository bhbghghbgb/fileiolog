use fileiolog::etw::EtwEvent;

#[derive(Debug, Clone, EtwEvent)]
pub struct BadEvent {
    #[etw_prop(name = "Value", convert_with = some_fn)]
    pub value: u64,
}

fn some_fn(v: u64) -> u64 { v }

fn main() {}
