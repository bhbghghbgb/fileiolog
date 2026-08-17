use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use ferrisetw::provider::kernel_providers::KernelProvider;
use ferrisetw::provider::Provider;
use ferrisetw::trace::{KernelTrace, TraceTrait, UserTrace, stop_trace_by_name};
use ferrisetw::{EventRecord, GUID, SchemaLocator};
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, WNODE_FLAG_TRACED_GUID,
};
use windows::core::PCWSTR;

use crate::config::{AppConfig, TraceMode};
use crate::output::OutputWriter;
use crate::tdh;

/// Run the trace session based on config
pub fn run_session(config: &AppConfig) -> Result<(), String> {
    match config.mode {
        TraceMode::Kernel => run_kernel_session(config),
        TraceMode::User => run_user_session(config),
    }
}

/// Run a kernel trace session
fn run_kernel_session(config: &AppConfig) -> Result<(), String> {
    let provider_guid = config.parse_provider_guid()?;
    let enable_flags = config.parse_enable_flags().unwrap_or(0);
    let group_mask = config.parse_group_mask();

    let session_name = "TdhEnumeratorKernel";
    let _ = stop_trace_by_name(session_name);

    let output_writer = Arc::new(OutputWriter::new(&config.output_prefix));
    let output_callback = output_writer.event_callback();

    // Create kernel provider with specified flags
    let kernel_provider = KernelProvider::new(provider_guid, enable_flags);

    let callback_output = output_callback.clone();
    let provider = Provider::kernel(&kernel_provider)
        .level(config.level)
        .any(0)
        .all(0)
        .add_callback(move |record: &EventRecord, _schema_locator: &SchemaLocator| {
            // Extract event info using TDH directly
            match tdh::extract_event_info(record) {
                Ok(info) => callback_output(info),
                Err(e) => {
                    log::warn!(
                        "Failed to extract event info: opcode={}, error={}",
                        record.opcode(),
                        e
                    );
                }
            }
        })
        .build();

    let (_trace, trace_handle) = KernelTrace::new()
        .named(session_name.to_string())
        .enable(provider)
        .stop_if_exist(true)
        .start()
        .map_err(|e| format!("KernelTrace start failed: {:?}", e))?;

    // Get control handle and set group mask if provided
    let control_handle = query_control_handle(session_name)
        .map_err(|e| format!("Failed to get control handle: {:?}", e))?;

    if let Some(mask) = group_mask {
        set_group_mask(control_handle, enable_flags, mask)?;
    }

    // Spawn processing thread
    let proc_handle = trace_handle;
    let thread = std::thread::spawn(move || {
        let _ = <KernelTrace as TraceTrait>::process_from_handle(proc_handle);
    });

    log::info!("Kernel trace session '{}' active for {} seconds...", session_name, config.duration);
    std::thread::sleep(Duration::from_secs(config.duration));

    // Stop the session
    stop_kernel_session(session_name, control_handle)?;

    // Wait for processing thread to finish
    let _ = thread.join();

    log::info!("Collected {} events", output_writer.event_count());

    // Write output files
    output_writer.write_files()?;

    Ok(())
}

/// Run a user trace session
fn run_user_session(config: &AppConfig) -> Result<(), String> {
    let provider_guid = config.parse_provider_guid()?;
    let keyword = config.parse_keyword();

    let session_name = "TdhEnumeratorUser";
    let _ = stop_trace_by_name(session_name);

    let output_writer = Arc::new(OutputWriter::new(&config.output_prefix));
    let output_callback = output_writer.event_callback();

    let callback_output = output_callback.clone();
    let provider = Provider::by_guid(provider_guid)
        .level(config.level)
        .any(keyword)
        .all(0)
        .add_callback(move |record: &EventRecord, _schema_locator: &SchemaLocator| {
            match tdh::extract_event_info(record) {
                Ok(info) => callback_output(info),
                Err(e) => {
                    log::warn!(
                        "Failed to extract event info: opcode={}, error={}",
                        record.opcode(),
                        e
                    );
                }
            }
        })
        .build();

    let (_trace, trace_handle) = UserTrace::new()
        .named(session_name.to_string())
        .enable(provider)
        .start()
        .map_err(|e| format!("UserTrace start failed: {:?}", e))?;

    let proc_handle = trace_handle;
    let thread = std::thread::spawn(move || {
        let _ = <UserTrace as TraceTrait>::process_from_handle(proc_handle);
    });

    log::info!("User trace session '{}' active for {} seconds...", session_name, config.duration);
    std::thread::sleep(Duration::from_secs(config.duration));

    // Stop the session
    let _ = stop_trace_by_name(session_name);

    let _ = thread.join();

    log::info!("Collected {} events", output_writer.event_count());

    // Write output files
    output_writer.write_files()?;

    Ok(())
}

/// Query control handle by session name
fn query_control_handle(session_name: &str) -> Result<CONTROLTRACE_HANDLE, String> {
    let name_wide: Vec<u16> = session_name.encode_utf16().chain(std::iter::once(0)).collect();
    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let name_buf_size = (200 + 1) * 2;
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
        ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
    }

    let result = unsafe {
        Etw::ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(name_ptr as *const u16),
            props as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    };

    result.ok().map_err(|e| format!("ControlTraceW QUERY failed: {:?}", e))?;

    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
    Ok(CONTROLTRACE_HANDLE {
        Value: handle_value,
    })
}

/// Set PERFINFO_GROUPMASK via TraceSetInformation
fn set_group_mask(
    control_handle: CONTROLTRACE_HANDLE,
    enable_flags: u32,
    masks: [u32; 8],
) -> Result<(), String> {
    let mut group_mask_data = masks;
    group_mask_data[0] |= enable_flags;

    const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;

    let result = unsafe {
        Etw::TraceSetInformation(
            control_handle,
            std::mem::transmute(TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO),
            group_mask_data.as_ptr() as *const std::ffi::c_void,
            std::mem::size_of::<[u32; 8]>() as u32,
        )
    };

    result.ok().map_err(|e| format!("TraceSetInformation (GroupMask) failed: {:?}", e))
}

/// Stop kernel session by sending CONTROL_TRACE_STOP
fn stop_kernel_session(
    session_name: &str,
    control_handle: CONTROLTRACE_HANDLE,
) -> Result<(), String> {
    let name_wide: Vec<u16> = session_name.encode_utf16().chain(std::iter::once(0)).collect();
    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let name_buf_size = (200 + 1) * 2;
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
        ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
    }

    let result = unsafe {
        Etw::ControlTraceW(
            control_handle,
            PCWSTR::from_raw(name_ptr as *const u16),
            props as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_STOP,
        )
    };

    result.ok().map_err(|e| format!("ControlTraceW STOP failed: {:?}", e))
}
