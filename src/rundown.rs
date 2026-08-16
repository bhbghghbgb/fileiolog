use std::ptr;

use windows::Win32::System::Diagnostics::Etw::{
    CONTROLTRACE_HANDLE, ControlTraceW, EVENT_CONTROL_CODE_CAPTURE_STATE,
    EVENT_TRACE_CONTROL_QUERY, EVENT_TRACE_PROPERTIES, EnableTraceEx2, TRACE_LEVEL_NONE,
    WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

/// Max session name length, matching ferrisetw's `TRACE_NAME_MAX_CHARS`.
const NAME_MAX: usize = 200;

/// Call `ControlTraceW(EVENT_TRACE_CONTROL_QUERY)` to retrieve the session's
/// control handle from `Wnode.HistoricalContext`.
pub fn query_control_handle(
    session_name: &str,
) -> Result<CONTROLTRACE_HANDLE, std::io::Error> {
    let name_wide: Vec<u16> = session_name.encode_utf16().collect();
    let name_len = name_wide.len().min(NAME_MAX);

    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let name_buf_size = (NAME_MAX + 1) * 2;
    let total_size = header_size + name_buf_size;

    let mut buffer = vec![0u8; total_size];

    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };

    props.Wnode.BufferSize = total_size as u32;
    props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    props.Wnode.Guid = GUID::zeroed();
    props.LoggerNameOffset = header_size as u32;
    props.LogFileNameOffset = 0;

    let name_ptr = unsafe { buffer.as_mut_ptr().add(header_size) as *mut u16 };
    unsafe {
        ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_len);
        ptr::write(name_ptr.add(name_len), 0);
    }

    let result = unsafe {
        ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(name_ptr as *const u16),
            props as *mut EVENT_TRACE_PROPERTIES,
            EVENT_TRACE_CONTROL_QUERY,
        )
    };

    let win32_result = result.ok();
    if let Err(e) = win32_result {
        return Err(map_win32_error(e, session_name));
    }

    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };

    Ok(CONTROLTRACE_HANDLE {
        Value: handle_value,
    })
}

/// Request rundown (DCStart/DCEnd) by calling `EnableTraceEx2(CAPTURE_STATE)`
/// for every provider GUID in `provider_guids` on the given control handle.
pub fn request_rundown(
    handle: CONTROLTRACE_HANDLE,
    provider_guids: &[GUID],
) -> Result<(), std::io::Error> {
    for &guid in provider_guids {
        trigger_capture_state(handle, guid)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  EnableTraceEx2(CAPTURE_STATE)
// ---------------------------------------------------------------------------
fn trigger_capture_state(
    handle: CONTROLTRACE_HANDLE,
    provider_guid: GUID,
) -> Result<(), std::io::Error> {
    let result = unsafe {
        EnableTraceEx2(
            handle,
            &provider_guid as *const GUID,
            EVENT_CONTROL_CODE_CAPTURE_STATE.0,
            TRACE_LEVEL_NONE as u8,
            0,    // match any keyword
            0,    // match all keyword
            0,    // timeout (0 → infinite)
            None, // enable parameters
        )
    };
    log::debug!("Triggered capture state for {provider_guid:?}");

    let win32_result = result.ok();
    if let Err(e) = win32_result {
        return Err(map_win32_error(e, &format!("{provider_guid:?}")));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
//  helpers
// ---------------------------------------------------------------------------
fn map_win32_error(e: windows::core::Error, context: &str) -> std::io::Error {
    let code = std::io::Error::from_raw_os_error(e.code().0);
    log::error!("ETW rundown call failed for `{context}`: {code:?}");
    code
}
