#![allow(dead_code)]
#![allow(unused_imports)]

/// Trait for parsing an ETW event struct from a ferrisetw parser.
///
/// Implemented by the `#[derive(EtwEvent)]` macro.
pub trait EtwEventParse: Sized {
    fn try_from_parser(
        parser: &ferrisetw::parser::Parser<'_, '_>,
    ) -> Result<Self, ferrisetw::parser::ParserError>;
}

/// Trait for converting an intermediate ETW parsed type to the desired field type.
///
/// Used by `#[etw_prop(name = "...", parse_as = IntermediateType)]` in the derive macro.
/// When `convert_with` is not specified, the macro generates a call to `EtwPropConvert::convert`.
///
/// # Example
///
/// ```ignore
/// impl EtwPropConvert<ferrisetw::parser::Pointer> for usize {
///     fn convert(value: ferrisetw::parser::Pointer) -> Self {
///         *value
///     }
/// }
/// ```
pub trait EtwPropConvert<T> {
    fn convert(value: T) -> Self;
}

// ── Built-in EtwPropConvert implementations ─────────────────────

impl EtwPropConvert<ferrisetw::parser::Pointer> for usize {
    fn convert(value: ferrisetw::parser::Pointer) -> Self {
        *value
    }
}

impl EtwPropConvert<ferrisetw::native::time::FileTime> for time::OffsetDateTime {
    fn convert(value: ferrisetw::native::time::FileTime) -> Self {
        value.as_date_time()
    }
}

impl EtwPropConvert<ferrisetw::native::time::SystemTime> for time::OffsetDateTime {
    fn convert(value: ferrisetw::native::time::SystemTime) -> Self {
        value.as_date_time()
    }
}

// Re-export the proc-macros so users can `use fileiolog::etw::...`.
pub use etw_macros::{EtwEvent, etw_provider, guid};
