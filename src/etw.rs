/// Trait for parsing an ETW event struct from a ferrisetw parser.
///
/// Implemented by the `#[derive(EtwEvent)]` macro.
pub trait EtwEventParse: Sized {
    fn try_from_parser(
        parser: &ferrisetw::parser::Parser<'_, '_>,
    ) -> Result<Self, ferrisetw::parser::ParserError>;
}

// Re-export the proc-macros so users can `use fileiolog::etw::...`.
pub use etw_macros::{etw_provider, EtwEvent};
