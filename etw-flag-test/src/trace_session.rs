use std::sync::{Arc, Mutex};

use ferrisetw::provider::kernel_providers;
use ferrisetw::{EventRecord, KernelTrace, SchemaLocator};
use ferrisetw::provider::Provider;
use ferrisetw::trace::TraceTrait;
use windows::Win32::System::Diagnostics::Etw::{
    self, CONTROLTRACE_HANDLE, EVENT_TRACE_PROPERTIES, WNODE_FLAG_TRACED_GUID,
};

use crate::flags::{self, TestConfig};

const NAME_MAX: usize = 200;
const SESSION_NAME: &str = "EtWFlagTest";

/// Shared state for collecting events during a test run.
#[derive(Debug, Default)]
pub struct EventCollector {
    pub seen: std::collections::HashSet<(u16, u8)>,
    pub total_count: u64,
}

impl EventCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_event(&mut self, id: u16, version: u8) {
        self.seen.insert((id, version));
        self.total_count += 1;
    }
}

/// Query ControlTraceW to get the session's control handle.
fn query_control_handle(session_name: &str) -> Result<CONTROLTRACE_HANDLE, std::io::Error> {
    let name_wide: Vec<u16> = session_name.encode_utf16().collect();
    let name_len = name_wide.len().min(NAME_MAX);

    let header_size = std::mem::size_of::<EVENT_TRACE_PROPERTIES>();
    let name_buf_size = (NAME_MAX + 1) * 2;
    let total_size = header_size + name_buf_size;

    let mut buffer = vec![0u8; total_size];

    let props = unsafe { &mut *(buffer.as_mut_ptr() as *mut EVENT_TRACE_PROPERTIES) };

    props.Wnode.BufferSize = total_size as u32;
    props.Wnode.Flags = WNODE_FLAG_TRACED_GUID;
    props.Wnode.Guid = windows::core::GUID::zeroed();
    props.LoggerNameOffset = header_size as u32;
    props.LogFileNameOffset = 0;

    let name_ptr = unsafe { buffer.as_mut_ptr().add(header_size) as *mut u16 };
    unsafe {
        std::ptr::copy_nonoverlapping(name_wide.as_ptr(), name_ptr, name_len);
        std::ptr::write(name_ptr.add(name_len), 0);
    }

    let result = unsafe {
        Etw::ControlTraceW(
            CONTROLTRACE_HANDLE { Value: 0 },
            windows::core::PCWSTR::from_raw(name_ptr as *const u16),
            props as *mut EVENT_TRACE_PROPERTIES,
            Etw::EVENT_TRACE_CONTROL_QUERY,
        )
    };

    result.ok().map_err(|e| {
        std::io::Error::from_raw_os_error(e.code().0)
    })?;

    let handle_value = unsafe { props.Wnode.Anonymous1.HistoricalContext };
    Ok(CONTROLTRACE_HANDLE {
        Value: handle_value,
    })
}

/// Set the PERFINFO_GROUPMASK on a running trace session via TraceSetInformation.
fn set_group_mask(
    control_handle: CONTROLTRACE_HANDLE,
    masks: &[u32; 8],
) -> Result<(), std::io::Error> {
    let result = unsafe {
        Etw::TraceSetInformation(
            control_handle,
            Etw::TraceSystemTraceEnableFlagsInfo,
            masks.as_ptr() as *const _,
            std::mem::size_of::<[u32; 8]>() as u32,
        )
    };

    result.ok().map_err(|e| {
        std::io::Error::from_raw_os_error(e.code().0)
    })
}

/// Perform basic file I/O operations to trigger FileIo events.
fn trigger_file_io() {
    let dir = std::env::temp_dir().join("etw_flag_test");
    let _ = std::fs::create_dir_all(&dir);

    let test_file = dir.join("test_trigger.tmp");

    // Create + Write + Flush + Close
    {
        use std::io::Write;
        if let Ok(mut f) = std::fs::File::create(&test_file) {
            let _ = f.write_all(b"hello etw");
            let _ = f.flush();
        }
    }

    // Read + Close
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open(&test_file) {
            let mut buf = [0u8; 64];
            let _ = f.read(&mut buf);
        }
    }

    // QueryInfo (stat)
    let _ = std::fs::metadata(&test_file);

    // SetInfo (truncate)
    {
        if let Ok(f) = std::fs::OpenOptions::new()
            .write(true)
            .open(&test_file)
        {
            let _ = f.set_len(0);
        }
    }

    // Rename
    let rename_target = dir.join("test_trigger_renamed.tmp");
    let _ = std::fs::rename(&test_file, &rename_target);

    // Delete
    let _ = std::fs::remove_file(&rename_target);

    // DirEnum (list dir)
    let _ = std::fs::read_dir(&dir);

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

/// Build the event callback closure that records events to the shared collector.
fn make_callback(
    collector: Arc<Mutex<EventCollector>>,
) -> impl FnMut(&EventRecord, &SchemaLocator) + Send + Sync + 'static {
    move |record: &EventRecord, _schema_locator: &SchemaLocator| {
        let id = record.event_id();
        let version = record.version();
        log::trace!("Event: id={} ver={}", id, version);
        if let Ok(mut coll) = collector.lock() {
            coll.record_event(id, version);
        }
    }
}

/// Run a single test configuration and return the set of event types seen.
pub fn run_single_test(
    config: &TestConfig,
    collector: Arc<Mutex<EventCollector>>,
) -> std::collections::HashSet<(u16, u8)> {
    // 1. Stop any existing session with our name
    let _ = ferrisetw::trace::stop_trace_by_name(SESSION_NAME);

    match config {
        TestConfig::EnableFlags(flags) => {
            run_enable_flags_test(*flags, collector)
        }
        TestConfig::GroupMask(masks) => {
            run_group_mask_test(masks, collector)
        }
    }
}

/// Test with EnableFlags: re-create the trace with the exact flags.
fn run_enable_flags_test(
    flags: u32,
    collector: Arc<Mutex<EventCollector>>,
) -> std::collections::HashSet<(u16, u8)> {
    // Map the flags to a kernel provider. For combined flags, we use
    // the broadest single provider and then apply the group mask.
    let provider = if flags == flags::enable_flags::EVENT_TRACE_FLAG_FILE_IO_INIT {
        Provider::kernel(&kernel_providers::FILE_INIT_IO_PROVIDER)
    } else if flags == flags::enable_flags::EVENT_TRACE_FLAG_FILE_IO {
        Provider::kernel(&kernel_providers::FILE_IO_PROVIDER)
    } else if flags == flags::enable_flags::EVENT_TRACE_FLAG_DISK_FILE_IO {
        Provider::kernel(&kernel_providers::DISK_FILE_IO_PROVIDER)
    } else if flags == flags::enable_flags::EVENT_TRACE_FLAG_VAMAP {
        Provider::kernel(&kernel_providers::VAMAP_PROVIDER)
    } else {
        // For combined flags, convert to group mask and use that path
        let mut masks = [0u32; 8];
        if flags & flags::enable_flags::EVENT_TRACE_FLAG_FILE_IO_INIT != 0 {
            flags::group_mask::set_mask(&mut masks, flags::group_mask::PERF_FILE_IO_INIT);
        }
        if flags & flags::enable_flags::EVENT_TRACE_FLAG_FILE_IO != 0 {
            flags::group_mask::set_mask(&mut masks, flags::group_mask::PERF_FILE_IO);
        }
        if flags & flags::enable_flags::EVENT_TRACE_FLAG_DISK_FILE_IO != 0 {
            flags::group_mask::set_mask(&mut masks, flags::group_mask::PERF_FILENAME);
        }
        if flags & flags::enable_flags::EVENT_TRACE_FLAG_VAMAP != 0 {
            flags::group_mask::set_mask(&mut masks, flags::group_mask::PERF_VAMAP);
        }
        return run_group_mask_test(&masks, collector);
    };

    let cb_collector = Arc::clone(&collector);
    let provider = provider.add_callback(make_callback(cb_collector)).build();

    let builder = KernelTrace::new()
        .named(String::from(SESSION_NAME))
        .enable(provider)
        .stop_if_exist(true);

    let (trace, trace_handle) = match builder.start() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to start trace: {:?}", e);
            return std::collections::HashSet::new();
        }
    };

    // Spawn processing thread
    std::thread::spawn(move || {
        let _ = KernelTrace::process_from_handle(trace_handle);
    });

    // Brief wait for trace to stabilize
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Trigger file I/O
    trigger_file_io();

    // Wait for events to arrive
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Stop the trace
    let _ = trace.stop();

    collector.lock().unwrap().seen.clone()
}

/// Test with PERFINFO_GROUPMASK: start with a minimal trace, then set the group mask.
fn run_group_mask_test(
    masks: &[u32; 8],
    collector: Arc<Mutex<EventCollector>>,
) -> std::collections::HashSet<(u16, u8)> {
    // Start with FILE_IO_INIT provider (broadest single flag)
    let cb_collector = Arc::clone(&collector);
    let provider = Provider::kernel(&kernel_providers::FILE_INIT_IO_PROVIDER)
        .add_callback(make_callback(cb_collector))
        .build();

    let (trace, trace_handle) = match KernelTrace::new()
        .named(String::from(SESSION_NAME))
        .enable(provider)
        .stop_if_exist(true)
        .start()
    {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to start trace: {:?}", e);
            return std::collections::HashSet::new();
        }
    };

    // Get the control handle
    let control_handle = match query_control_handle(SESSION_NAME) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to query control handle: {:?}", e);
            let _ = trace.stop();
            return std::collections::HashSet::new();
        }
    };

    // Set the group mask
    if let Err(e) = set_group_mask(control_handle, masks) {
        log::error!("Failed to set group mask: {:?}", e);
        let _ = trace.stop();
        return std::collections::HashSet::new();
    }
    log::debug!("Group mask applied: {:?}", masks);

    // Spawn processing thread
    std::thread::spawn(move || {
        let _ = KernelTrace::process_from_handle(trace_handle);
    });

    // Brief wait for trace to stabilize
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Trigger file I/O
    trigger_file_io();

    // Wait for events to arrive
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Stop the trace
    let _ = trace.stop();

    collector.lock().unwrap().seen.clone()
}
