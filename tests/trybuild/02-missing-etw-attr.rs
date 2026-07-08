use fileiolog::etw::EtwEvent;

#[derive(Debug, Clone, EtwEvent)]
pub struct BadEvent {
    #[etw(prop = "Value")]
    pub value: u64,
    pub name: String,
}

fn main() {}
