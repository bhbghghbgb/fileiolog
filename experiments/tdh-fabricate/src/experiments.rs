//! Individual experiments for understanding TdhGetEventInformation requirements.
//!
//! Each experiment is a standalone function that tests a specific hypothesis
//! about what fields in EVENT_RECORD are needed for the TDH API to succeed.

use ferrisetw::EventRecord;
use ferrisetw::provider::kernel_providers::KernelProvider;
use ferrisetw::provider::Provider;
use ferrisetw::trace::{KernelTrace, TraceTrait, stop_trace_by_name};
use ferrisetw::GUID;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::Win32::System::Diagnostics::Etw::EVENT_RECORD;

use crate::fabricate::{
    FabricatedRecord, FILEIO_PROVIDER_GUID, KERNEL_PROCESS_MANIFEST_GUID, KERNEL_FILEIO_MANIFEST_GUID,
};
use crate::tdh_helpers::{call_tdh_get_event_information, TdhResult};

/// A captured event: raw EVENT_RECORD bytes + user data hex
#[derive(Clone)]
struct CapturedEvent {
    record_bytes: Vec<u8>,
    user_data_hex: String,
}

/// Helper: create a FabricatedRecord from a CapturedEvent
fn fab_from_captured(captured: &CapturedEvent) -> FabricatedRecord {
    FabricatedRecord::from_bytes(&captured.record_bytes)
}

/// Helper: get raw EVENT_RECORD reference from captured bytes
unsafe fn captured_as_raw(captured: &CapturedEvent) -> &EVENT_RECORD {
    unsafe { &*(captured.record_bytes.as_ptr() as *const EVENT_RECORD) }
}

/// Helper: create EventRecord reference from captured bytes
unsafe fn captured_as_event_record(captured: &CapturedEvent) -> &EventRecord {
    unsafe { &*(captured.record_bytes.as_ptr() as *const EVENT_RECORD as *const EventRecord) }
}

/// Helper: run a kernel trace for a short duration and collect the first N events.
/// Returns empty vec if kernel tracing is not available (non-admin).
fn capture_kernel_events(
    provider_guid: GUID,
    enable_flags: u32,
    group_mask: Option<[u32; 8]>,
    duration_secs: u64,
    max_events: usize,
) -> Vec<CapturedEvent> {
    let session_name = "TdhFabricateCapture";
    let _ = stop_trace_by_name(session_name);

    let captured: Arc<Mutex<Vec<CapturedEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured.clone();

    let cb = move |record: &EventRecord, _schema_locator: &ferrisetw::SchemaLocator| {
        let mut vec = captured_clone.lock().unwrap();
        if vec.len() >= max_events {
            return;
        }

        let raw = unsafe { &*(record as *const EventRecord as *const EVENT_RECORD) };
        let record_bytes = unsafe {
            std::slice::from_raw_parts(
                raw as *const EVENT_RECORD as *const u8,
                std::mem::size_of::<EVENT_RECORD>(),
            )
        }
        .to_vec();

        let user_data_hex = if !raw.UserData.is_null() && raw.UserDataLength > 0 {
            let slice = unsafe {
                std::slice::from_raw_parts(raw.UserData as *const u8, raw.UserDataLength as usize)
            };
            hex::encode(slice)
        } else {
            String::new()
        };

        vec.push(CapturedEvent {
            record_bytes,
            user_data_hex,
        });
    };

    let kernel_provider = KernelProvider::new(provider_guid, enable_flags);
    let provider = Provider::kernel(&kernel_provider)
        .level(0xFF)
        .any(0)
        .all(0)
        .add_callback(cb)
        .build();

    let (_trace, trace_handle) = match KernelTrace::new()
        .named(session_name.to_string())
        .enable(provider)
        .stop_if_exist(true)
        .start()
    {
        Ok(t) => t,
        Err(e) => {
            log::warn!("Failed to start kernel trace (need admin?): {:?}", e);
            return Vec::new();
        }
    };

    if let Some(mask) = group_mask {
        if let Ok(handle) = query_control_handle(session_name) {
            let _ = set_group_mask(handle, enable_flags, mask);
        }
    }

    let proc_handle = trace_handle;
    let thread = std::thread::spawn(move || {
        let _ = <KernelTrace as TraceTrait>::process_from_handle(proc_handle);
    });

    // Trigger file operations to generate ETW events for the trace session
    log::info!("Triggering file operations for kernel capture...");
    let bin = file_ops_trigger::bin_path();
    let _ = std::process::Command::new(&bin)
        .output()
        .map_err(|e| log::warn!("Failed to invoke file-ops-trigger: {}", e));

    std::thread::sleep(Duration::from_secs(duration_secs));
    let _ = stop_trace_by_name(session_name);
    let _ = thread.join();

    captured.lock().unwrap().clone()
}

/// Helper: query control handle for a running trace
fn query_control_handle(
    session_name: &str,
) -> Result<windows::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE, String> {
    use std::ptr;
    use windows::Win32::System::Diagnostics::Etw::{
        self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, WNODE_FLAG_TRACED_GUID,
    };
    use windows::core::PCWSTR;

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

    result
        .ok()
        .map_err(|e| format!("ControlTraceW QUERY failed: {:?}", e))?;

    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
    Ok(CONTROLTRACE_HANDLE {
        Value: handle_value,
    })
}

/// Helper: set PERFINFO_GROUPMASK
fn set_group_mask(
    control_handle: windows::Win32::System::Diagnostics::Etw::CONTROLTRACE_HANDLE,
    enable_flags: u32,
    masks: [u32; 8],
) -> Result<(), String> {
    use windows::Win32::System::Diagnostics::Etw;

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

// ═══════════════════════════════════════════════════════════════
// Experiment 1: Baseline
// ═══════════════════════════════════════════════════════════════

pub fn experiment_1_baseline() {
    println!();
    println!("============================================================");
    println!("Experiment 1: Baseline - Real Events + TDH");
    println!("============================================================");

    let events = capture_kernel_events(
        FILEIO_PROVIDER_GUID,
        0x04000000, // EVENT_TRACE_FLAG_FILE_IO_INIT
        None,
        2,
        5,
    );

    println!("Captured {} events", events.len());

    for (i, captured) in events.iter().enumerate() {
        let record = unsafe { captured_as_event_record(captured) };
        let result = call_tdh_get_event_information(record);

        println!(
            "  Event {}: id={}, version={}, opcode={} -> {}",
            i,
            record.event_id(),
            record.version(),
            record.opcode(),
            result.summary()
        );

        if !captured.user_data_hex.is_empty() {
            let preview = if captured.user_data_hex.len() > 40 {
                format!("{}...", &captured.user_data_hex[..40])
            } else {
                captured.user_data_hex.clone()
            };
            println!(
                "    UserData ({} bytes): {}",
                captured.user_data_hex.len() / 2,
                preview
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment 2: Minimal Fabrication
// ═══════════════════════════════════════════════════════════════

pub fn experiment_2_minimal_fabrication() {
    println!();
    println!("============================================================");
    println!("Experiment 2: Minimal Fabrication (ProviderId + EventDescriptor only)");
    println!("============================================================");

    let test_cases: Vec<(&str, u16, u8, u8, u16, u64)> = vec![
        ("FileIo_Create_v2", 64, 2, 0, 0, 0),
        ("FileIo_Cleanup_v2", 65, 2, 0, 0, 0),
        ("FileIo_Close_v2", 66, 2, 0, 0, 0),
        ("FileIo_Read_v2", 67, 2, 0, 0, 0),
        ("FileIo_Write_v2", 68, 2, 0, 0, 0),
        ("FileIo_Create_v3", 64, 3, 0, 0, 0),
        ("FileIo_Name_v2_id0", 0, 2, 0, 0, 0),
        ("FileIo_Name_v2_id32", 32, 2, 0, 0, 0),
        ("FileIo_OpEnd_v2", 76, 2, 0, 0, 0),
        ("FltIo_PreOpInit_v3", 96, 3, 0, 0, 0),
        ("FltIo_PostOpInit_v3", 97, 3, 0, 0, 0),
        ("FltIo_PreOpCompletion_v3", 98, 3, 0, 0, 0),
        ("FltIo_PostOpCompletion_v3", 99, 3, 0, 0, 0),
        ("FltIo_PreOpFailure_v3", 100, 3, 0, 0, 0),
        ("FltIo_PostOpFailure_v3", 101, 3, 0, 0, 0),
    ];

    // First: test with zero flags (as we have been doing)
    println!("  --- With Flags=0 (default) ---");
    for (name, id, version, opcode, task, keyword) in &test_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(FILEIO_PROVIDER_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, *task, *keyword);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }

    // Second: test with CLASSIC_HEADER flag (for MOF-based providers)
    // Legacy kernel providers use MOF classes, not XML manifests
    println!();
    println!("  --- With CLASSIC_HEADER flag (0x0010) ---");
    for (name, id, version, opcode, task, keyword) in &test_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(FILEIO_PROVIDER_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, *task, *keyword);
        fab.set_flags(0x0010); // EVENT_HEADER_FLAG_CLASSIC_HEADER

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }

    // Third: test with LEGACY_EVENTLOG property
    println!();
    println!("  --- With LEGACY_EVENTLOG property (0x0004) ---");
    for (name, id, version, opcode, task, keyword) in &test_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(FILEIO_PROVIDER_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, *task, *keyword);
        fab.set_event_property(0x0004); // EVENT_HEADER_PROPERTY_LEGACY_EVENTLOG

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }

    // Fourth: test with EXTENDED_INFO flag
    println!();
    println!("  --- With EXTENDED_INFO flag (0x0001) ---");
    for (name, id, version, opcode, task, keyword) in &test_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(FILEIO_PROVIDER_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, *task, *keyword);
        fab.set_flags(0x0001); // EVENT_HEADER_FLAG_EXTENDED_INFO

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }

    // Fifth: test with non-zero EventId (real events use non-zero IDs)
    println!();
    println!("  --- Combining CLASSIC_HEADER + real opcode values ---");
    let real_opcode_cases: Vec<(&str, u16, u8, u8)> = vec![
        // opcode from MOF definitions: 0=FileIo, 32=FileCreate, etc.
        ("Create(64,v2,op=0)", 64, 2, 0),
        ("Create(64,v2,op=64)", 64, 2, 64),
        ("Cleanup(65,v2,op=0)", 65, 2, 0),
        ("Cleanup(65,v2,op=65)", 65, 2, 65),
        ("Name(0,v2,op=0)", 0, 2, 0),
        ("Name(0,v2,op=36)", 0, 2, 36),
    ];

    for (name, id, version, opcode) in &real_opcode_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(FILEIO_PROVIDER_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, 0, 0);
        fab.set_flags(0x0010); // CLASSIC_HEADER

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} -> {}",
            status, name, result.summary()
        );
    }

    // Sixth: test with manifest-based providers (user-mode, registered in WINEVT)
    // These providers have XML manifests and should be findable by TDH
    println!();
    println!("  --- Manifest-based providers (Microsoft-Windows-Kernel-Process) ---");
    let manifest_test_cases: Vec<(&str, GUID, u16, u8, u8)> = vec![
        // Process Start event (id=1, version=1)
        ("ProcessStart", KERNEL_PROCESS_MANIFEST_GUID, 1, 1, 0),
        // Process Stop event (id=2, version=1)
        ("ProcessStop", KERNEL_PROCESS_MANIFEST_GUID, 2, 1, 0),
        // Thread Start (id=3, version=1)
        ("ThreadStart", KERNEL_PROCESS_MANIFEST_GUID, 3, 1, 0),
        // Thread Stop (id=4, version=1)
        ("ThreadStop", KERNEL_PROCESS_MANIFEST_GUID, 4, 1, 0),
    ];

    for (name, guid, id, version, opcode) in &manifest_test_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(*guid);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, 0, 0);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }

    // Seventh: test Microsoft-Windows-Kernel-FileIo (user-mode manifest version)
    println!();
    println!("  --- Manifest-based FileIo (Microsoft-Windows-Kernel-FileIo) ---");
    let fileio_manifest_cases: Vec<(&str, u16, u8, u8)> = vec![
        ("FileCreate", 12, 0, 0),
        ("FileCleanup", 13, 0, 0),
        ("FileClose", 14, 0, 0),
        ("FileRead", 15, 0, 0),
        ("FileWrite", 16, 0, 0),
        ("FileNameCreate", 10, 0, 0),
        ("FileNameDelete", 11, 0, 0),
        ("OperationEnd", 24, 0, 0),
    ];

    for (name, id, version, opcode) in &fileio_manifest_cases {
        let mut fab = FabricatedRecord::new();
        fab.set_provider_id(KERNEL_FILEIO_MANIFEST_GUID);
        fab.set_descriptor(*id, *version, 0, 0xFF, *opcode, 0, 0);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK" } else { "FAIL" };
        println!(
            "  [{:>4}] {} (id={}, v{}, op={}) -> {}",
            status, name, id, version, opcode, result.summary()
        );
    }
}
// ═══════════════════════════════════════════════════════════════

pub fn experiment_3_field_sensitivity() {
    println!();
    println!("============================================================");
    println!("Experiment 3: Field Sensitivity Analysis");
    println!("============================================================");

    let events = capture_kernel_events(
        FILEIO_PROVIDER_GUID,
        0x04000000, // EVENT_TRACE_FLAG_FILE_IO_INIT
        None,
        2,
        1,
    );

    if events.is_empty() {
        println!("  No events captured - skipping. Run with admin privileges.");
        return;
    }

    let captured = &events[0];
    let baseline_record = unsafe { captured_as_event_record(captured) };
    let baseline_result = call_tdh_get_event_information(baseline_record);
    println!(
        "  Baseline: id={}, v{}, op={} -> {}",
        baseline_record.event_id(),
        baseline_record.version(),
        baseline_record.opcode(),
        baseline_result.summary()
    );

    if !baseline_result.is_success() {
        println!("  Baseline failed - cannot proceed.");
        return;
    }

    let fields_to_test: Vec<(&str, fn(&mut FabricatedRecord))> = vec![
        ("Zero ProviderId", |fab| {
            fab.set_provider_id(GUID::zeroed());
        }),
        ("Zero EventId", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(0).write(0);
                p.add(1).write(0);
            }
        }),
        ("Zero Version", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(2).write(0);
            }
        }),
        ("Zero Opcode", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(5).write(0);
            }
        }),
        ("Zero Task", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(6).write(0);
                p.add(7).write(0);
            }
        }),
        ("Zero Keyword", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                for i in 8..16 {
                    p.add(i).write(0);
                }
            }
        }),
        ("Zero Flags", |fab| {
            fab.set_flags(0);
        }),
        ("Zero EventProperty", |fab| {
            fab.set_event_property(0);
        }),
        ("Zero BufferContext", |fab| {
            fab.set_buffer_context(0, 0);
        }),
        ("Zero ProcessId + ThreadId", |fab| {
            fab.set_process_thread(0, 0);
        }),
        ("Zero TimeStamp", |fab| {
            fab.set_timestamp(0);
        }),
        ("Zero HeaderSize", |fab| {
            fab.raw_record_mut().EventHeader.Size = 0;
        }),
    ];

    for (field_name, zero_fn) in &fields_to_test {
        let mut fab = fab_from_captured(captured);
        zero_fn(&mut fab);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK " } else { "FAIL" };
        let detail = if result.is_success() {
            "still works".to_string()
        } else {
            result.summary()
        };
        println!("  [{:>4}] {} -> {}", status, field_name, detail);
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment 4: Version Probing
// ═══════════════════════════════════════════════════════════════

pub fn experiment_4_version_probing() {
    println!();
    println!("============================================================");
    println!("Experiment 4: Version Probing");
    println!("============================================================");

    let event_ids: Vec<(&str, u16, Vec<u8>)> = vec![
        ("FileIo_Create", 64, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_Cleanup", 65, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_Close", 66, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_Read", 67, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_Write", 68, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_Name_id0", 0, vec![0, 1, 2, 3, 4, 5]),
        ("FileIo_OpEnd", 76, vec![0, 1, 2, 3, 4, 5]),
        ("FltIo_PreOpInit", 96, vec![0, 1, 2, 3, 4, 5]),
        ("FltIo_PostOpFailure", 101, vec![0, 1, 2, 3, 4, 5]),
    ];

    for (name, id, versions) in &event_ids {
        println!("  {} (id={}):", name, id);
        for version in versions {
            let mut fab = FabricatedRecord::new();
            fab.set_provider_id(FILEIO_PROVIDER_GUID);
            fab.set_descriptor(*id, *version, 0, 0xFF, 0, 0, 0);

            let record = unsafe { fab.as_event_record() };
            let result = call_tdh_get_event_information(record);

            let status = if result.is_success() { "OK " } else { "FAIL" };
            let detail = match &result {
                TdhResult::Success {
                    opcode_name,
                    property_count,
                    ..
                } => format!(
                    "opcode_name={}, props={}",
                    opcode_name, property_count
                ),
                _ => result.summary(),
            };
            println!("    v{} -> [{:>4}] {}", version, status, detail);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment 5: Modify Real Record
// ═══════════════════════════════════════════════════════════════

pub fn experiment_5_modify_real_record() {
    println!();
    println!("============================================================");
    println!("Experiment 5: Modify Real Record Fields");
    println!("============================================================");

    let events = capture_kernel_events(
        FILEIO_PROVIDER_GUID,
        0x04000000,
        None,
        2,
        1,
    );

    if events.is_empty() {
        println!("  No events captured - skipping. Run with admin privileges.");
        return;
    }

    let captured = &events[0];
    let baseline_record = unsafe { captured_as_event_record(captured) };
    println!(
        "  Baseline: id={}, v{}, op={}",
        baseline_record.event_id(),
        baseline_record.version(),
        baseline_record.opcode()
    );

    let modifications: Vec<(&str, fn(&mut FabricatedRecord))> = vec![
        ("Change Id to 65 (Cleanup)", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(0).write(65);
                p.add(1).write(0);
            }
        }),
        ("Change Version to 3", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(2).write(3);
            }
        }),
        ("Change Version to 99 (nonexistent)", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(2).write(99);
            }
        }),
        ("Change Opcode to 1", |fab| {
            let raw = fab.raw_record_mut();
            let p = &mut raw.EventHeader.EventDescriptor as *mut _ as *mut u8;
            unsafe {
                p.add(5).write(1);
            }
        }),
        ("Change ProviderId to empty GUID", |fab| {
            fab.set_provider_id(GUID::zeroed());
        }),
        ("Change ProviderId to Process GUID", |fab| {
            fab.set_provider_id(crate::fabricate::PROCESS_PROVIDER_GUID);
        }),
    ];

    for (mod_name, mod_fn) in &modifications {
        let mut fab = fab_from_captured(captured);
        mod_fn(&mut fab);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK " } else { "FAIL" };
        let detail = match &result {
            TdhResult::Success {
                opcode_name,
                property_count,
                ..
            } => format!(
                "opcode_name={}, props={}",
                opcode_name, property_count
            ),
            _ => result.summary(),
        };
        println!("  [{:>4}] {} -> {}", status, mod_name, detail);
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment 6: Flags and EventProperty Effects
// ═══════════════════════════════════════════════════════════════

pub fn experiment_6_flags_and_properties() {
    println!();
    println!("============================================================");
    println!("Experiment 6: Flags and EventProperty Effects");
    println!("============================================================");

    let events = capture_kernel_events(
        FILEIO_PROVIDER_GUID,
        0x04000000,
        None,
        2,
        1,
    );

    if events.is_empty() {
        println!("  No events captured - skipping. Run with admin privileges.");
        return;
    }

    let captured = &events[0];
    let baseline_record = unsafe { captured_as_event_record(captured) };
    let baseline_raw = unsafe { captured_as_raw(captured) };
    println!(
        "  Baseline: Flags=0x{:04x}, EventProperty=0x{:04x}",
        baseline_record.event_flags(),
        baseline_raw.EventHeader.EventProperty,
    );

    // EVENT_HEADER_FLAG values
    const FLAG_EXTENDED_INFO: u16 = 0x0001;
    const FLAG_TRACE_MESSAGE: u16 = 0x0002;
    const FLAG_CLASSIC_HEADER: u16 = 0x0010;
    const FLAG_INDIRECT_TRACE: u16 = 0x0040;

    // EVENT_HEADER_PROPERTY values
    const PROP_XML: u16 = 0x0001;
    const PROP_FORWARDED_XML: u16 = 0x0002;
    const PROP_LEGACY_EVENTLOG: u16 = 0x0004;

    let flag_tests: Vec<(&str, u16)> = vec![
        ("Flags=0 (none)", 0),
        ("EXTENDED_INFO (0x0001)", FLAG_EXTENDED_INFO),
        ("TRACE_MESSAGE (0x0002) WPP", FLAG_TRACE_MESSAGE),
        ("CLASSIC_HEADER (0x0010)", FLAG_CLASSIC_HEADER),
        ("INDIRECT_TRACE (0x0040)", FLAG_INDIRECT_TRACE),
    ];

    for (flag_name, flags) in &flag_tests {
        let mut fab = fab_from_captured(captured);
        fab.set_flags(*flags);
        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);
        let status = if result.is_success() { "OK " } else { "FAIL" };
        println!("  [{:>4}] {} -> {}", status, flag_name, result.summary());
    }

    println!();

    let property_tests: Vec<(&str, u16)> = vec![
        ("EventProperty=0 (none)", 0),
        ("XML (0x0001)", PROP_XML),
        ("FORWARDED_XML (0x0002)", PROP_FORWARDED_XML),
        ("LEGACY_EVENTLOG (0x0004)", PROP_LEGACY_EVENTLOG),
    ];

    for (prop_name, prop) in &property_tests {
        let mut fab = fab_from_captured(captured);
        fab.set_event_property(*prop);
        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);
        let status = if result.is_success() { "OK " } else { "FAIL" };
        println!("  [{:>4}] {} -> {}", status, prop_name, result.summary());
    }
}

// ═══════════════════════════════════════════════════════════════
// Experiment 7: UserData Effects
// ═══════════════════════════════════════════════════════════════

pub fn experiment_7_userdata_effects() {
    println!();
    println!("============================================================");
    println!("Experiment 7: UserData Effects on Schema Lookup");
    println!("============================================================");

    let events = capture_kernel_events(
        FILEIO_PROVIDER_GUID,
        0x04000000,
        None,
        2,
        1,
    );

    if events.is_empty() {
        println!("  No events captured - skipping. Run with admin privileges.");
        return;
    }

    let captured = &events[0];
    let baseline_raw = unsafe { captured_as_raw(captured) };
    println!(
        "  Baseline: UserDataLength={}",
        baseline_raw.UserDataLength
    );

    if !captured.user_data_hex.is_empty() {
        let real_data = hex::decode(&captured.user_data_hex).unwrap_or_default();
        let preview = if real_data.len() > 20 {
            hex::encode(&real_data[..20])
        } else {
            captured.user_data_hex.clone()
        };
        println!("  Real UserData ({} bytes): {}...", real_data.len(), preview);
    }

    let userdata_tests: Vec<(&str, Vec<u8>)> = vec![
        ("Empty UserData (0 bytes)", vec![]),
        ("8 bytes zeros", vec![0u8; 8]),
        ("16 bytes zeros", vec![0u8; 16]),
        ("32 bytes zeros", vec![0u8; 32]),
        ("64 bytes zeros", vec![0u8; 64]),
        ("256 bytes zeros", vec![0u8; 256]),
        ("1024 bytes zeros", vec![0u8; 1024]),
    ];

    for (test_name, data) in &userdata_tests {
        let mut fab = fab_from_captured(captured);
        let leaked = data.clone().leak();
        fab.set_user_data(leaked);

        let record = unsafe { fab.as_event_record() };
        let result = call_tdh_get_event_information(record);

        let status = if result.is_success() { "OK " } else { "FAIL" };
        let detail = if result.is_success() {
            "still works".to_string()
        } else {
            result.summary()
        };
        println!(
            "  [{:>4}] {} ({} bytes) -> {}",
            status,
            test_name,
            data.len(),
            detail
        );
    }
}
