mod events;
mod trace_session;
mod file_ops;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use events::{FileIoEvent, EVENT_REGISTRY};
use trace_session::{KernelTraceSession, TraceConfig};

/// Test configuration for a single flag/mask combination
struct TestConfig {
    name: String,
    enable_flags: Option<u32>,
    group_mask: Option<[u32; 8]>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== FileIo ETW Trace Test ===");
    log::info!("This test will iterate through different EnableFlags and PERFINFO_GROUPMASK");
    log::info!("configurations to discover which FileIo event types are enabled by each.");
    log::info!("");

    // Define all test configurations
    let test_configs = build_test_configs();

    log::info!("Total configurations to test: {}", test_configs.len());
    log::info!("");

    // Shared storage for results
    let results: Arc<Mutex<HashMap<String, Vec<FileIoEvent>>>> = Arc::new(Mutex::new(HashMap::new()));

    // Print all known events at startup
    log::trace!("Known FileIo events:");
    for event in EVENT_REGISTRY.values() {
        log::trace!("  {:?}", event);
    }

    // Run tests sequentially
    for (i, config) in test_configs.iter().enumerate() {
        log::info!("");
        log::info!("=== Test {}/{}: {} ===", i + 1, test_configs.len(), config.name);
        log::info!("  EnableFlags: {:?}", config.enable_flags.map(|f| format!("0x{:08X}", f)));
        log::info!("  GroupMask: {:?}", config.group_mask.map(|m| {
            format!("[{:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}]",
                m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7])
        }));

        let collected_events = run_single_test(config);

        // Store results
        {
            let mut results = results.lock().unwrap();
            results.insert(config.name.clone(), collected_events.clone());
        }

        // Print discovered events
        log::info!("  Discovered {} events:", collected_events.len());
        let mut event_counts: HashMap<(u16, u8), usize> = HashMap::new();
        for event in &collected_events {
            *event_counts.entry((event.event_id, event.version)).or_insert(0) += 1;
        }
        for ((id, version), count) in &event_counts {
            if let Some(known) = EVENT_REGISTRY.get(&(*id, *version)) {
                log::info!("    ID={}, Version={}, Class={}, Name={}, Count={}",
                    id, version, known.class_name, known.event_name, count);
            } else {
                log::warn!("    ID={}, Version={} (UNKNOWN EVENT), Count={}", id, version, count);
            }
        }

        // Brief pause between tests
        if i < test_configs.len() - 1 {
            log::info!("  Pausing 2 seconds before next test...");
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    // Print summary
    log::info!("");
    log::info!("=== SUMMARY ===");
    print_summary(&results.lock().unwrap());
}

/// Build all test configurations (EnableFlags and PERFINFO_GROUPMASK)
fn build_test_configs() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    // --- EnableFlags-based tests ---

    // EVENT_TRACE_FLAG_DISK_FILE_IO (0x00000200)
    // Official docs say: requires DISK_IO, enables FileIo_Name
    configs.push(TestConfig {
        name: "EVENT_TRACE_FLAG_DISK_FILE_IO".to_string(),
        enable_flags: Some(0x00000200),
        group_mask: None,
    });

    // EVENT_TRACE_FLAG_FILE_IO (0x02000000)
    // Official docs say: enables FileIo_OpEnd
    configs.push(TestConfig {
        name: "EVENT_TRACE_FLAG_FILE_IO".to_string(),
        enable_flags: Some(0x02000000),
        group_mask: None,
    });

    // EVENT_TRACE_FLAG_FILE_IO_INIT (0x04000000)
    // Official docs say: enables Create, DirEnum, Info, ReadWrite, SimpleOp
    configs.push(TestConfig {
        name: "EVENT_TRACE_FLAG_FILE_IO_INIT".to_string(),
        enable_flags: Some(0x04000000),
        group_mask: None,
    });

    // Combination: FILE_IO_INIT + FILE_IO
    configs.push(TestConfig {
        name: "FILE_IO_INIT + FILE_IO".to_string(),
        enable_flags: Some(0x04000000 | 0x02000000),
        group_mask: None,
    });

    // Combination: All three FileIo flags
    configs.push(TestConfig {
        name: "DISK_FILE_IO + FILE_IO + FILE_IO_INIT".to_string(),
        enable_flags: Some(0x00000200 | 0x02000000 | 0x04000000),
        group_mask: None,
    });

    // --- PERFINFO_GROUPMASK-based tests ---
    // These use the undocumented extended mask mechanism

    // PERF_FLT_IO_INIT (0x80080000) - FltIoInit events
    configs.push(TestConfig {
        name: "PERF_FLT_IO_INIT".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80080000)),
    });

    // PERF_FLT_IO (0x80100000) - FltIoCompletion events
    configs.push(TestConfig {
        name: "PERF_FLT_IO".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80100000)),
    });

    // PERF_FLT_FASTIO (0x80200000) - FastIO events
    configs.push(TestConfig {
        name: "PERF_FLT_FASTIO".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80200000)),
    });

    // PERF_FLT_IO_FAILURE (0x80400000) - FltIoFailure events
    configs.push(TestConfig {
        name: "PERF_FLT_IO_FAILURE".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80400000)),
    });

    // All FLT masks combined
    configs.push(TestConfig {
        name: "ALL_FLT_MASKS".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80080000 | 0x80100000 | 0x80200000 | 0x80400000)),
    });

    // Combination: FILE_IO_INIT + FLT masks
    configs.push(TestConfig {
        name: "FILE_IO_INIT + ALL_FLT_MASKS".to_string(),
        enable_flags: Some(0x04000000),
        group_mask: Some(build_group_mask(0x80080000 | 0x80100000 | 0x80200000 | 0x80400000)),
    });

    // Comprehensive test: all FileIo flags + all FLT masks
    configs.push(TestConfig {
        name: "ALL_FILEIO_FLAGS + ALL_FLT_MASKS".to_string(),
        enable_flags: Some(0x00000200 | 0x02000000 | 0x04000000),
        group_mask: Some(build_group_mask(0x80080000 | 0x80100000 | 0x80200000 | 0x80400000)),
    });

    configs
}

/// Build a PERFINFO_GROUPMASK from a combined mask value
/// The mask value has the group index encoded in the high 3 bits
fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    let group_index = ((mask_value >> 29) & 0x07) as usize;
    masks[group_index] = mask_value;
    masks
}

/// Run a single test configuration
fn run_single_test(config: &TestConfig) -> Vec<FileIoEvent> {
    let collected_events: Arc<Mutex<Vec<FileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let session_name = format!("FileIoTest-{}", config.name.replace(" ", "_").replace("+", "_"));

    // Build trace configuration
    let trace_config = TraceConfig {
        session_name: session_name.clone(),
        enable_flags: config.enable_flags,
        group_mask: config.group_mask,
    };

    // Create and start the trace session
    let mut session = match KernelTraceSession::new(trace_config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create trace session: {:?}", e);
            return Vec::new();
        }
    };

    // Start the trace and get the process handle
    let _trace_handle = match session.start() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start trace: {:?}", e);
            return Vec::new();
        }
    };

    // Spawn processing thread
    let process_handle = session.get_trace_handle();
    let processing_thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(process_handle);
    });

    // Wait for the trace to stabilize
    std::thread::sleep(Duration::from_millis(500));

    // Trigger file system events
    log::info!("  Triggering file system events...");
    file_ops::trigger_all_file_operations();

    // Wait for events to be collected
    log::info!("  Waiting for events to arrive (5 seconds)...");
    std::thread::sleep(Duration::from_secs(5));

    // Request rundown (DCEnd events)
    log::info!("  Requesting rundown...");
    if let Err(e) = session.request_rundown() {
        log::warn!("  Rundown request failed: {:?}", e);
    }

    // Wait for rundown events
    std::thread::sleep(Duration::from_secs(2));

    // Stop the trace and collect events
    log::info!("  Stopping trace...");
    if let Err(e) = session.stop() {
        log::error!("  Failed to stop trace: {:?}", e);
    }

    // Wait for processing thread to finish
    let _ = processing_thread.join();

    // Return collected events
    let events = collected_events.lock().unwrap().clone();
    events
}

/// Print summary of all test results
fn print_summary(results: &HashMap<String, Vec<FileIoEvent>>) {
    // Collect all unique event types seen across all tests
    let mut all_events: HashMap<(u16, u8), Vec<String>> = HashMap::new();

    for (config_name, events) in results {
        for event in events {
            let key = (event.event_id, event.version);
            all_events.entry(key).or_default().push(config_name.clone());
        }
    }

    log::info!("");
    log::info!("Event-to-Flag/Mask Mapping:");
    log::info!("==========================");

    // Sort by event ID and version
    let mut sorted_events: Vec<_> = all_events.into_iter().collect();
    sorted_events.sort_by_key(|((id, ver), _)| (*id, *ver));

    for ((id, version), config_names) in &sorted_events {
        if let Some(known) = EVENT_REGISTRY.get(&(*id, *version)) {
            log::info!("Event: {} (ID={}, Version={})", known.event_name, id, version);
            log::info!("  Class: {}", known.class_name);
            log::info!("  Enabled by:");
            for name in config_names {
                log::info!("    - {}", name);
            }
        } else {
            log::warn!("Unknown Event: ID={}, Version={}", id, version);
            log::warn!("  Enabled by:");
            for name in config_names {
                log::warn!("    - {}", name);
            }
        }
    }
}
