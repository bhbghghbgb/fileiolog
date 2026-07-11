#![allow(unused_imports)]
use fileiolog::etw::EtwEvent;

#[derive(Debug, Clone, EtwEvent)]
pub struct TupleEvent(u64, String);

fn main() {}
