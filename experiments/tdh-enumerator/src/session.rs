use std::collections::{HashMap, HashSet};
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
use crate::output::{DiskWriter, WriteCommand};
use crate::tdh;
use crate::types::{EventError, EventTypeId};

/// Run the trace session based on config
pub fn run_session(config: &AppConfig) -> Result<(), String> {
    match config.mode {
        TraceMode::Kernel => run_kernel_session(config),
        TraceMode::User => run_user_session(config),
    }
}

/// Shared callback construction for both kernel and user sessions.
///
/// The callback:
/// 1. Builds a lightweight EventTypeId (no TDH)
/// 2. Checks the seen_types cache
/// 3. On first appearance: extracts full schema via TDH, sends schema + observation
/// 4. On subsequent appearances: sends only observation
/// 5. Logs TDH extraction errors only once per event key
fn build_callback(
    seen_types: Arc<std::sync::RwLock<HashMap<EventTypeId, u32>>>,
    disk_writer: Arc<DiskWriter>,
    logged_errors: Arc<std::sync::RwLock<HashSet<String>>>,
) -> impl Fn(&EventRecord, &SchemaLocator) + Clone + Send + Sync + 'static {
    move |record: &EventRecord, _schema_locator: &SchemaLocator| {
        let type_id = tdh::build_type_id(record);

        // Fast path: check if we've seen this event type before
        {
            let seen = seen_types.read().unwrap();
            if let Some(_count) = seen.get(&type_id) {
                // Already seen - just emit a lightweight observation (no TDH calls)
                let key = format!(
                    "{}_{}_v{}_op{}",
                    tdh::sanitize_key_static(&type_id.provider_guid),
                    type_id.event_id,
                    type_id.version,
                    type_id.opcode,
                );
                let obs = tdh::build_observation(record, &key);
                disk_writer.send(WriteCommand::Observation(obs));
                return;
            }
        }

        // First appearance - extract full schema (expensive TDH call)
        match tdh::extract_event_type_info(record) {
            Ok(type_info) => {
                let key = format!(
                    "{}_{}_v{}_op{}",
                    tdh::sanitize_key_static(&type_id.provider_guid),
                    type_id.event_id,
                    type_id.version,
                    type_id.opcode,
                );

                // Log first appearance
                log::info!("New event type: {}", type_info);

                // Mark as seen
                {
                    let mut seen = seen_types.write().unwrap();
                    seen.insert(type_id, 1);
                }

                // Send schema and observation to disk writer
                disk_writer.send(WriteCommand::NewType(type_info));
                let obs = tdh::build_observation(record, &key);
                disk_writer.send(WriteCommand::Observation(obs));
            }
            Err(e) => {
                let key = format!(
                    "{}_{}_v{}_op{}",
                    tdh::sanitize_key_static(&type_id.provider_guid),
                    type_id.event_id,
                    type_id.version,
                    type_id.opcode,
                );

                // Still record the observation from the EVENT_RECORD
                let obs = tdh::build_observation(record, &key);
                disk_writer.send(WriteCommand::Observation(obs));

                // Send error to disk and log (once per event key)
                let mut errors = logged_errors.write().unwrap();
                if errors.insert(key.clone()) {
                    let event_err = EventError {
                        type_key: key.clone(),
                        error_message: e.clone(),
                    };
                    disk_writer.send(WriteCommand::Error(event_err));
                    log::warn!("Failed to extract event info: key={}, error={}", key, e);
                }
            }
        }
    }
}

/// Run a kernel trace session
fn run_kernel_session(config: &AppConfig) -> Result<(), String> {
    let provider_guid = config.parse_provider_guid()?;
    let enable_flags = config.parse_enable_flags().unwrap_or(0);
    let group_mask = config.parse_group_mask();

    let session_name = "TdhEnumeratorKernel";
    let _ = stop_trace_by_name(session_name);

    // Pre-allocate seen_types with reasonable capacity
    let seen_types: Arc<std::sync::RwLock<HashMap<EventTypeId, u32>>> =
        Arc::new(std::sync::RwLock::new(HashMap::with_capacity(256)));
    let logged_errors: Arc<std::sync::RwLock<HashSet<String>>> =
        Arc::new(std::sync::RwLock::new(HashSet::new()));
    let disk_writer = Arc::new(DiskWriter::new(&config.output_prefix));

    let kernel_provider = KernelProvider::new(provider_guid, enable_flags);

    let cb = build_callback(seen_types.clone(), disk_writer.clone(), logged_errors.clone());
    let provider = Provider::kernel(&kernel_provider)
        .level(config.level)
        .any(0)
        .all(0)
        .add_callback(cb)
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

    log::info!(
        "Kernel trace session '{}' active for {} seconds...",
        session_name,
        config.duration
    );

    // Trigger file operations if requested
    if config.trigger {
        log::info!("Triggering file operations...");
        crate::file_ops::trigger_all_file_operations();
        log::info!("File operations completed");
    }

    std::thread::sleep(Duration::from_secs(config.duration));

    // Stop the session
    stop_kernel_session(session_name, control_handle)?;

    // Wait for processing thread to finish
    let _ = thread.join();

    // Drop the trace so the provider (and its callback holding disk_writer) is released
    drop(_trace);

    // Report stats
    let type_count = seen_types.read().unwrap().len();
    log::info!("Observed {} distinct event types", type_count);

    // Shutdown disk writer (writes final summary)
    Arc::try_unwrap(disk_writer)
        .ok()
        .expect("Arc should be unique at shutdown")
        .shutdown();

    Ok(())
}

/// Run a user trace session
fn run_user_session(config: &AppConfig) -> Result<(), String> {
    let provider_guid = config.parse_provider_guid()?;
    let keyword = config.parse_keyword();

    let session_name = "TdhEnumeratorUser";
    let _ = stop_trace_by_name(session_name);

    let seen_types: Arc<std::sync::RwLock<HashMap<EventTypeId, u32>>> =
        Arc::new(std::sync::RwLock::new(HashMap::with_capacity(256)));
    let logged_errors: Arc<std::sync::RwLock<HashSet<String>>> =
        Arc::new(std::sync::RwLock::new(HashSet::new()));
    let disk_writer = Arc::new(DiskWriter::new(&config.output_prefix));

    let cb = build_callback(seen_types.clone(), disk_writer.clone(), logged_errors.clone());
    let provider = Provider::by_guid(provider_guid)
        .level(config.level)
        .any(keyword)
        .all(0)
        .add_callback(cb)
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

    log::info!(
        "User trace session '{}' active for {} seconds...",
        session_name,
        config.duration
    );

    // Trigger file operations if requested
    if config.trigger {
        log::info!("Triggering file operations...");
        crate::file_ops::trigger_all_file_operations();
        log::info!("File operations completed");
    }

    std::thread::sleep(Duration::from_secs(config.duration));

    // Stop the session
    let _ = stop_trace_by_name(session_name);
    let _ = thread.join();

    // Drop the trace so the provider (and its callback holding disk_writer) is released
    drop(_trace);

    let type_count = seen_types.read().unwrap().len();
    log::info!("Observed {} distinct event types", type_count);

    Arc::try_unwrap(disk_writer)
        .ok()
        .expect("Arc should be unique at shutdown")
        .shutdown();

    Ok(())
}

/// Query control handle by session name
fn query_control_handle(session_name: &str) -> Result<CONTROLTRACE_HANDLE, String> {
    let name_wide: Vec<u16> = session_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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

    result
        .ok()
        .map_err(|e| format!("TraceSetInformation (GroupMask) failed: {:?}", e))
}

/// Stop kernel session by sending CONTROL_TRACE_STOP
fn stop_kernel_session(
    session_name: &str,
    control_handle: CONTROLTRACE_HANDLE,
) -> Result<(), String> {
    let name_wide: Vec<u16> = session_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
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
