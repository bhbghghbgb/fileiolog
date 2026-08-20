//! TDH (Trace Data Helper) API wrapper functions.
//!
//! Wraps TdhGetEventInformation and provides result types for experiment use.

use std::alloc::Layout;

use ferrisetw::EventRecord;
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::System::Diagnostics::Etw::{self, TRACE_EVENT_INFO};

/// Result of calling TdhGetEventInformation
#[derive(Debug)]
pub enum TdhResult {
    /// Success - contains the decoded schema info
    Success {
        provider_name: String,
        task_name: String,
        opcode_name: String,
        decoding_source: String,
        property_count: u32,
    },
    /// API returned an error code
    Error {
        error_code: u32,
        error_name: String,
    },
    /// Buffer was zero-sized (should not happen)
    ZeroBuffer,
}

impl TdhResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TdhResult::Success { .. })
    }

    pub fn summary(&self) -> String {
        match self {
            TdhResult::Success {
                provider_name,
                opcode_name,
                property_count,
                decoding_source,
                ..
            } => {
                format!(
                    "OK: provider={}, opcode={}, props={}, source={}",
                    provider_name, opcode_name, property_count, decoding_source
                )
            }
            TdhResult::Error {
                error_code,
                error_name,
            } => {
                format!("ERR({}): {}", error_code, error_name)
            }
            TdhResult::ZeroBuffer => "ERR: zero buffer size".into(),
        }
    }
}

/// Error code to name mapping
fn error_code_name(code: u32) -> String {
    match code {
        0 => "ERROR_SUCCESS".into(),
        2 => "ERROR_FILE_NOT_FOUND".into(),
        8 => "ERROR_NOT_ENOUGH_MEMORY".into(),
        13 => "ERROR_INVALID_DATA".into(),
        4317 => "ERROR_NOT_FOUND".into(),
        111 => "ERROR_ALREADY_EXISTS".into(),
        87 => "ERROR_INVALID_PARAMETER".into(),
        1223 => "ERROR_CANCELLED".into(),
        _ => format!("UNKNOWN({})", code),
    }
}

/// Decode a TRACE_EVENT_INFO buffer into a TdhResult
fn decode_trace_event_info(data: *const u8, te_info: &TRACE_EVENT_INFO) -> TdhResult {
    let provider_name = extract_string(data, te_info.ProviderNameOffset);
    let task_name = extract_string(data, te_info.TaskNameOffset);
    let opcode_name = extract_string(data, te_info.OpcodeNameOffset);

    let decoding_source = match te_info.DecodingSource.0 {
        0 => "XML File".into(),
        1 => "WMI MOF".into(),
        2 => "WPP".into(),
        3 => "TraceLogging".into(),
        v => format!("Unknown({})", v),
    };

    TdhResult::Success {
        provider_name,
        task_name,
        opcode_name,
        decoding_source,
        property_count: te_info.PropertyCount,
    }
}

/// Call TdhGetEventInformation on an EventRecord reference.
///
/// This is the core TDH API call used by ferrisetw's SchemaLocator.
pub fn call_tdh_get_event_information(record: &EventRecord) -> TdhResult {
    let raw_ptr = unsafe {
        &*(record as *const EventRecord
            as *const windows::Win32::System::Diagnostics::Etw::EVENT_RECORD)
    };

    // First call: get required buffer size
    let mut buffer_size: u32 = 0;
    let status = unsafe { Etw::TdhGetEventInformation(raw_ptr, None, None, &mut buffer_size) };

    if status != ERROR_INSUFFICIENT_BUFFER.0 {
        return TdhResult::Error {
            error_code: status,
            error_name: error_code_name(status),
        };
    }

    if buffer_size == 0 {
        return TdhResult::ZeroBuffer;
    }

    // Allocate buffer
    let layout = match Layout::from_size_align(buffer_size as usize, 8) {
        Ok(l) => l,
        Err(e) => {
            return TdhResult::Error {
                error_code: 0,
                error_name: format!("Layout error: {}", e),
            };
        }
    };

    let data = unsafe { std::alloc::alloc(layout) };
    if data.is_null() {
        return TdhResult::Error {
            error_code: 8,
            error_name: "Allocation failed".into(),
        };
    }

    // Second call: get the actual data
    let status = unsafe {
        Etw::TdhGetEventInformation(
            raw_ptr,
            None,
            Some(data as *mut TRACE_EVENT_INFO),
            &mut buffer_size,
        )
    };

    if status != 0 {
        unsafe {
            std::alloc::dealloc(data, layout);
        }
        return TdhResult::Error {
            error_code: status,
            error_name: error_code_name(status),
        };
    }

    // Decode the result
    let te_info = unsafe { &*(data as *const TRACE_EVENT_INFO) };
    let result = decode_trace_event_info(data, te_info);

    // Free the buffer
    unsafe {
        std::alloc::dealloc(data, layout);
    }

    result
}

/// Extract a UTF-16 string from a TRACE_EVENT_INFO buffer at a given offset
fn extract_string(data_ptr: *const u8, offset: u32) -> String {
    if offset == 0 {
        return String::new();
    }
    let ptr = unsafe { data_ptr.offset(offset as isize) };
    if ptr.is_null() {
        return String::new();
    }
    let wide_ptr = ptr as *const u16;
    unsafe {
        let mut len = 0;
        while *wide_ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(wide_ptr, len);
        String::from_utf16_lossy(slice)
    }
}
