//! ETW session running for the FLT comparison.
//!
//! Provides:
//!  * `run_single`  – one session enabled with a specific group mask.
//!  * `run_dual`    – two concurrent sessions, each with its own group mask,
//!                    receiving the *same* live kernel event stream. One
//!                    shared workload is triggered while both are active.
//!
//! Each session enables the kernel FileIo provider and sets its extended
//! group mask via `TraceSetInformation` (`PERFINFO_GROUPMASK`). Events are
//! captured with their raw kernel timestamp (identical across sessions for
//! the same underlying event), pid, tid, opcode/version, and a payload
//! signature.

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use ferrisetw::EventRecord;
use ferrisetw::provider::*;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::*;
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, PROCESSTRACE_HANDLE, WNODE_FLAG_TRACED_GUID,
};
use windows::core::{GUID, PCWSTR};

use crate::flt_events::{FltEvent, major_function, payload_sig};

/// FileIo provider GUID.
pub const FILE_IO_GUID: GUID = GUID::from_u128(0x90cbdc39_4a3e_11d1_84f4_0000f80464e3);

/// PERFINFO_GROUPMASK extended flags (group index 4).
pub const PERF_FLT_IO: u32 = 0x80100000;
pub const PERF_FLT_FASTIO: u32 = 0x80200000;

fn group_index(mask_value: u32) -> usize {
    ((mask_value >> 29) & 0x07) as usize
}

fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    masks[group_index(mask_value)] = mask_value;
    masks
}

/// A single captured event from a session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapturedEvent {
    pub opcode: u8,
    pub version: u8,
    /// Raw kernel timestamp (100ns). Identical between concurrent sessions for
    /// the same underlying kernel event.
    pub timestamp: u64,
    pub pid: u32,
    pub tid: u32,
    /// Payload signature for FltIo init/completion/failure events (96-101).
    /// 0 if not parsed.
    pub sig: u64,
    pub major: u32,
    pub parse_ok: bool,
}

impl CapturedEvent {
    pub fn is_flt(&self) -> bool {
        (96..=101).contains(&self.opcode)
    }
}

fn capture(record: &EventRecord, schema_locator: &SchemaLocator) -> CapturedEvent {
    let opcode = record.opcode();
    let version = record.version();
    let timestamp = record.raw_timestamp() as u64;
    let pid = record.process_id();
    let tid = record.thread_id();

    let mut sig: u64 = 0;
    let mut major: u32 = 0;
    let mut parse_ok = false;
    if (96..=101).contains(&opcode) {
        if let Some(ev) = FltEvent::try_parse(record, schema_locator) {
            sig = payload_sig(&ev);
            major = major_function(&ev);
            parse_ok = true;
        }
    }

    CapturedEvent {
        opcode,
        version,
        timestamp,
        pid,
        tid,
        sig,
        major,
        parse_ok,
    }
}

/// A started kernel trace session.
#[allow(dead_code)]
pub struct StartedSession {
    #[allow(dead_code)]
    session: KernelTrace,
    trace_handle: PROCESSTRACE_HANDLE,
    control_handle: CONTROLTRACE_HANDLE,
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

/// Start one session with the given group mask.
pub fn start_session(name: &str, group_mask_value: u32) -> Result<StartedSession, String> {
    let collected: Arc<Mutex<Vec<CapturedEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = collected.clone();

    let kernel_provider = kernel_providers::KernelProvider::new(FILE_IO_GUID, 0);
    let provider = Provider::kernel(&kernel_provider)
        .level(0xFF)
        .any(0)
        .all(0)
        .add_callback(
            move |record: &EventRecord, schema_locator: &SchemaLocator| {
                let ev = capture(record, schema_locator);
                if let Ok(mut events) = events_clone.lock() {
                    events.push(ev);
                }
            },
        )
        .build();

    let builder = KernelTrace::new()
        .named(name.to_string())
        .enable(provider)
        .stop_if_exist(true);

    let (trace, trace_handle) = builder
        .start()
        .map_err(|e| format!("Trace '{}' start failed: {:?}", name, e))?;

    let control_handle = query_control_handle(name)?;
    set_group_mask(control_handle, build_group_mask(group_mask_value))
        .map_err(|e| format!("set_group_mask('{}') failed: {}", name, e))?;

    Ok(StartedSession {
        session: trace,
        trace_handle,
        control_handle,
        events: collected,
    })
}

impl StartedSession {
    /// Start dispatching (ProcessTrace) on a background thread.
    pub fn begin_dispatch(&self) -> JoinHandle<()> {
        let handle = self.trace_handle;
        std::thread::spawn(move || {
            use ferrisetw::trace::TraceTrait;
            let _ = <KernelTrace as TraceTrait>::process_from_handle(handle);
        })
    }

    /// Send CONTROL_STOP via the control handle, let the dispatch thread drain
    /// and exit, and return the collected events. The `KernelTrace` object is
    /// dropped by the caller afterwards (its own stop becomes a harmless no-op).
    pub fn stop(&mut self, thread: JoinHandle<()>) -> Vec<CapturedEvent> {
        stop_control(self.control_handle);
        std::thread::sleep(Duration::from_millis(400));
        let events = self.events.lock().map(|g| g.clone()).unwrap_or_default();
        let _ = thread.join();
        events
    }
}

/// Send CONTROL_STOP to a session by its control handle (best-effort).
pub fn stop_control(control_handle: CONTROLTRACE_HANDLE) {
    let mut buffer = build_control_properties();
    let res = unsafe {
        Etw::ControlTraceW(
            control_handle,
            PCWSTR::null(),
            buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_STOP,
        )
    };
    if res.0 != 0 {
        log::warn!("ControlTraceW STOP failed with code 0x{:X}", res.0);
    }
}

/// Run a single session trace (baseline pass). Triggers the workload if asked.
pub fn run_single(
    session_name: &str,
    group_mask_value: u32,
    trigger: bool,
) -> Vec<CapturedEvent> {
    let mut sess = match start_session(session_name, group_mask_value) {
        Ok(s) => s,
        Err(e) => {
            log::error!("{}", e);
            return Vec::new();
        }
    };
    let thread = sess.begin_dispatch();
    std::thread::sleep(Duration::from_millis(600));
    if trigger {
        log::info!("  Triggering file-system workload...");
        crate::file_ops::trigger_workload();
    }
    std::thread::sleep(Duration::from_secs(5));
    sess.stop(thread)
}

/// Run two concurrent sessions with different group masks over a shared
/// workload. Returns (events_A, events_B) filtered to FltIo events if
/// `filter_flt_only`.
pub fn run_dual(
    name_a: &str,
    mask_a: u32,
    name_b: &str,
    mask_b: u32,
    filter_flt_only: bool,
) -> (Vec<CapturedEvent>, Vec<CapturedEvent>) {
    let mut a = match start_session(name_a, mask_a) {
        Ok(s) => s,
        Err(e) => {
            log::error!("{}", e);
            return (Vec::new(), Vec::new());
        }
    };
    let mut b = match start_session(name_b, mask_b) {
        Ok(s) => s,
        Err(e) => {
            log::error!("{}", e);
            stop_control(a.control_handle);
            return (Vec::new(), Vec::new());
        }
    };

    let thread_a = a.begin_dispatch();
    let thread_b = b.begin_dispatch();

    // Stability window before triggering load.
    std::thread::sleep(Duration::from_millis(600));

    log::info!("  Triggering file-system workload...");
    crate::file_ops::trigger_workload();

    // Collect while events drain.
    std::thread::sleep(Duration::from_secs(5));

    let events_a = a.stop(thread_a);
    let events_b = b.stop(thread_b);

    log::info!(
        "  {}: {} raw events, {}: {} raw events",
        name_a,
        events_a.len(),
        name_b,
        events_b.len()
    );

    if filter_flt_only {
        (
            events_a.into_iter().filter(|e| e.is_flt()).collect(),
            events_b.into_iter().filter(|e| e.is_flt()).collect(),
        )
    } else {
        (events_a, events_b)
    }
}

/// Minimal `EVENT_TRACE_PROPERTIES` buffer for handle-based CONTROL calls.
fn build_control_properties() -> Vec<u8> {
    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let total_size = header_size + 8;
    let mut buffer = vec![0u8; total_size];
    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
    props.Wnode.BufferSize = total_size as u32;
    props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    props.Wnode.Guid = GUID::zeroed();
    props.LoggerNameOffset = 0;
    props.LogFileNameOffset = 0;
    buffer
}

/// Build an `EVENT_TRACE_PROPERTIES` buffer populated with the session name
/// (needed for name-based QUERY calls).
fn build_named_properties(name: &str) -> Vec<u8> {
    let name_wide: Vec<u16> = name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let total_size = header_size + name_wide.len() * 2;
    let mut buffer = vec![0u8; total_size];

    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
    props.Wnode.BufferSize = total_size as u32;
    props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    props.Wnode.Guid = GUID::zeroed();
    props.LoggerNameOffset = header_size as u32;
    props.LogFileNameOffset = 0;

    let name_ptr = unsafe { buffer.as_mut_ptr().add(header_size) as *mut u16 };
    unsafe {
        std::ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_wide.len());
    }
    buffer
}

/// Query a session's control handle by name (ControlTraceW QUERY).
fn query_control_handle(name: &str) -> Result<CONTROLTRACE_HANDLE, String> {
    let mut buffer = build_named_properties(name);
    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };
    let name_ptr =
        unsafe { buffer.as_mut_ptr().add(props.LoggerNameOffset as usize) as *const u16 };

    let result = unsafe {
        Etw::ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            PCWSTR::from_raw(name_ptr),
            props as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    };
    if result.0 != 0 {
        return Err(format!(
            "ControlTraceW QUERY failed for '{}' (code 0x{:X})",
            name, result.0
        ));
    }
    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
    Ok(CONTROLTRACE_HANDLE { Value: handle_value })
}

/// Set the extended `PERFINFO_GROUPMASK` via `TraceSystemTraceEnableFlagsInfo`.
fn set_group_mask(control_handle: CONTROLTRACE_HANDLE, masks: [u32; 8]) -> Result<(), String> {
    const TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO: i32 = 4;
    let result = unsafe {
        Etw::TraceSetInformation(
            control_handle,
            std::mem::transmute(TRACE_SYSTEM_TRACE_ENABLE_FLAGS_INFO),
            masks.as_ptr() as *const std::ffi::c_void,
            std::mem::size_of::<[u32; 8]>() as u32,
        )
    };
    if let Err(e) = result {
        return Err(format!("TraceSetInformation (GroupMask) failed: {:?}", e));
    }
    log::debug!("Set PERFINFO_GROUPMASK = {:?}", masks);
    Ok(())
}