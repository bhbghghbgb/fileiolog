pub use anyhow::Error as EtwError;

/// Trait for parsing an ETW event struct from a ferrisetw parser.
///
/// Implemented by the `#[derive(EtwEvent)]` macro.
pub trait EtwEventParse: Sized {
    fn try_from_parser(
        parser: &ferrisetw::parser::Parser<'_, '_>,
    ) -> Result<Self, EtwError>;
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
///     fn convert(value: ferrisetw::parser::Pointer) -> Result<Self, EtwError> {
///         Ok(*value)
///     }
/// }
/// ```
pub trait EtwPropConvert<T>: Sized {
    fn convert(value: T) -> Result<Self, EtwError>;
}

// ── Built-in EtwPropConvert implementations ─────────────────────

impl EtwPropConvert<ferrisetw::parser::Pointer> for usize {
    fn convert(value: ferrisetw::parser::Pointer) -> Result<Self, EtwError> {
        Ok(*value)
    }
}

/// Convert a ferrisetw `ParserError` into `EtwError` so it can be used with `?`.
pub fn parser_err_to_anyhow(e: ferrisetw::parser::ParserError) -> EtwError {
    EtwError::msg(format!("{:?}", e))
}

// Re-export the proc-macros so users can `use fileiolog::etw::...`.
pub use etw_macros::{etw_provider, EtwEvent};
