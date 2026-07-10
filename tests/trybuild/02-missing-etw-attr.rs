use fileiolog::etw::EtwEvent;

#[derive(Debug, Clone, EtwEvent)]
pub struct BadEvent {
    #[etw_prop(name = "Value")]
    pub value: u64,
    pub name: String,
}

fn main() {}
