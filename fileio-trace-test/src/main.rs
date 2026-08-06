mod events;
mod file_ops;
mod fileio_events;
mod persist;
mod trace_session;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use events::{EVENT_REGISTRY, ParsedFileIoEvent};
use trace_session::{KernelTraceSession, TraceConfig};

const PERSIST_FILE: &str = "fileio_test_results.json";

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

    // Load previous persisted results
    let persist_path = Path::new(PERSIST_FILE);
    let mut persisted = persist::load(persist_path);

    // Define all test configurations
    let test_configs = build_test_configs();

    log::info!("Total configurations to test: {}", test_configs.len());
    log::info!("");

    // Shared storage for raw events (needed for the original per-config display)
    let results: Arc<Mutex<HashMap<String, Vec<events::FileIoRawEvent>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Shared storage for parsed events (for comparison)
    let parsed_results: Arc<Mutex<HashMap<String, Vec<ParsedFileIoEvent>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Per-config event counts for this run (for merging)
    let mut current_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();

    // Print all known events at startup
    log::trace!("Known FileIo events:");
    for event in EVENT_REGISTRY.values() {
        log::trace!("  {:?}", event);
    }

    // Run tests sequentially
    for (i, config) in test_configs.iter().enumerate() {
        log::info!("");
        log::info!(
            "=== Test {}/{}: {} ===",
            i + 1,
            test_configs.len(),
            config.name
        );
        log::info!(
            "  EnableFlags: {:?}",
            config.enable_flags.map(|f| format!("0x{:08X}", f))
        );
        log::info!(
            "  GroupMask: {:?}",
            config.group_mask.map(|m| {
                format!(
                    "[{:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}]",
                    m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7]
                )
            })
        );

        let (collected_events, parsed_events) = run_single_test(config);

        // Store raw results
        {
            let mut results = results.lock().unwrap();
            results.insert(config.name.clone(), collected_events.clone());
        }

        // Store parsed results
        {
            let mut parsed = parsed_results.lock().unwrap();
            parsed.insert(config.name.clone(), parsed_events.clone());
        }

        // Compute and store this config's event counts for merging
        let counts = persist::compute_counts(&collected_events);
        current_counts.insert(config.name.clone(), counts.clone());

        // Print discovered events
        log::info!("  Discovered {} events:", collected_events.len());
        let mut event_counts: HashMap<(u8, u8), usize> = HashMap::new();
        for event in &collected_events {
            *event_counts
                .entry((event.opcode, event.version))
                .or_insert(0) += 1;
        }
        let mut sorted_counts: Vec<_> = event_counts.into_iter().collect();
        sorted_counts.sort_by_key(|((op, ver), _)| (*op, *ver));
        for ((opcode, version), count) in &sorted_counts {
            if let Some(known) = EVENT_REGISTRY.get(&(*opcode, *version)) {
                log::info!(
                    "    Opcode={}, Version={}, Class={}, Name={}, Count={}",
                    opcode,
                    version,
                    known.class_name,
                    known.event_name,
                    count
                );
            } else {
                log::warn!(
                    "    Opcode={}, Version={} (UNKNOWN EVENT), Count={}",
                    opcode,
                    version,
                    count
                );
            }
        }

        // Print parsed events for this config
        if !parsed_events.is_empty() {
            log::info!("  Parsed events:");
            for event in &parsed_events {
                if let Some(known) = EVENT_REGISTRY.get(&(event.opcode, event.version)) {
                    log::info!(
                        "    {} [{}] PID={} TID={} data={:?}",
                        known.event_name,
                        known.class_name,
                        event.process_id,
                        event.thread_id,
                        event.event
                    );
                }
            }
        }

        // Brief pause between tests
        if i < test_configs.len() - 1 {
            log::info!("  Pausing 2 seconds before next test...");
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    // Merge current run with persisted data and save
    persist::merge(&mut persisted, &current_counts);
    persist::save(persist_path, &persisted);

    // Display cumulative results
    persist::display(&persisted, &current_counts);

    // Display parsed event comparison
    display_parsed_comparison(&parsed_results);
}

/// Build all test configurations (EnableFlags and PERFINFO_GROUPMASK)
fn build_test_configs() -> Vec<TestConfig> {
    let mut configs = Vec::new();

    // ========================================================================
    // Group A: EnableFlags-based tests (EVENT_TRACE_FLAG_* constants)
    // ========================================================================

    // EVENT_TRACE_FLAG_DISK_FILE_IO (0x00000200)
    // Official docs say: requires DISK_IO, enables FileIo_Name
    configs.push(TestConfig {
        name: "EF:DISK_FILE_IO".to_string(),
        enable_flags: Some(0x00000200),
        group_mask: None,
    });

    // EVENT_TRACE_FLAG_FILE_IO (0x02000000)
    // Official docs say: enables FileIo_OpEnd
    configs.push(TestConfig {
        name: "EF:FILE_IO".to_string(),
        enable_flags: Some(0x02000000),
        group_mask: None,
    });

    // EVENT_TRACE_FLAG_FILE_IO_INIT (0x04000000)
    // Official docs say: enables Create, DirEnum, Info, ReadWrite, SimpleOp
    configs.push(TestConfig {
        name: "EF:FILE_IO_INIT".to_string(),
        enable_flags: Some(0x04000000),
        group_mask: None,
    });

    // EVENT_TRACE_FLAG_VAMAP (0x00008000)
    // Enables MapFile events (V2+)
    configs.push(TestConfig {
        name: "EF:VAMAP".to_string(),
        enable_flags: Some(0x00008000),
        group_mask: None,
    });

    // ========================================================================
    // Group B: PERFINFO_GROUPMASK tests — Masks[0] PERF_ equivalents
    // Same numerical values as EnableFlags, but set via groupmask.
    // Tests whether the mechanism matters or just the bit position.
    // ========================================================================

    // PERF_FILENAME (0x00000200) — same value as EVENT_TRACE_FLAG_DISK_FILE_IO
    configs.push(TestConfig {
        name: "GM:PERF_FILENAME".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x00000200)),
    });

    // PERF_FILE_IO (0x02000000) — same value as EVENT_TRACE_FLAG_FILE_IO
    configs.push(TestConfig {
        name: "GM:PERF_FILE_IO".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x02000000)),
    });

    // PERF_FILE_IO_INIT (0x04000000) — same value as EVENT_TRACE_FLAG_FILE_IO_INIT
    configs.push(TestConfig {
        name: "GM:PERF_FILE_IO_INIT".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x04000000)),
    });

    // PERF_VAMAP (0x00008000) — same value as EVENT_TRACE_FLAG_VAMAP
    configs.push(TestConfig {
        name: "GM:PERF_VAMAP".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x00008000)),
    });

    // ========================================================================
    // Group C: PERFINFO_GROUPMASK tests — Masks[4] extended masks
    // These have no EnableFlags equivalent; only accessible via groupmask.
    // ========================================================================

    // PERF_FLT_IO_INIT (0x80080000) — FltIoInit events
    configs.push(TestConfig {
        name: "GM:PERF_FLT_IO_INIT".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80080000)),
    });

    // PERF_FLT_IO (0x80100000) — FltIoCompletion events
    configs.push(TestConfig {
        name: "GM:PERF_FLT_IO".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80100000)),
    });

    // PERF_FLT_FASTIO (0x80200000) — FastIO events
    configs.push(TestConfig {
        name: "GM:PERF_FLT_FASTIO".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80200000)),
    });

    // PERF_FLT_IO_FAILURE (0x80400000) — FltIoFailure events
    configs.push(TestConfig {
        name: "GM:PERF_FLT_IO_FAILURE".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80400000)),
    });

    // ========================================================================
    // Group D: Combination tests
    // ========================================================================

    // All FileIo EnableFlags combined
    configs.push(TestConfig {
        name: "COMBO:ALL_EF_FLAGS".to_string(),
        enable_flags: Some(0x00000200 | 0x02000000 | 0x04000000 | 0x00008000),
        group_mask: None,
    });

    // All FLT masks combined (extended masks only)
    configs.push(TestConfig {
        name: "COMBO:ALL_FLT_MASKS".to_string(),
        enable_flags: None,
        group_mask: Some(build_group_mask(
            0x80080000 | 0x80100000 | 0x80200000 | 0x80400000,
        )),
    });

    // Everything combined: all EnableFlags + all FLT masks
    configs.push(TestConfig {
        name: "COMBO:ALL_EF+ALL_FLT".to_string(),
        enable_flags: Some(0x00000200 | 0x02000000 | 0x04000000 | 0x00008000),
        group_mask: Some(build_group_mask(
            0x80080000 | 0x80100000 | 0x80200000 | 0x80400000,
        )),
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
fn run_single_test(config: &TestConfig) -> (Vec<events::FileIoRawEvent>, Vec<ParsedFileIoEvent>) {
    let collected_events: Arc<Mutex<Vec<events::FileIoRawEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let parsed_events: Arc<Mutex<Vec<ParsedFileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let session_name = format!(
        "FileIoTest-{}",
        config.name
            .replace(" ", "_")
            .replace("+", "_")
            .replace(":", "_")
    );

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
            return (Vec::new(), Vec::new());
        }
    };

    // Start the trace and get the process handle
    let _trace_handle = match session.start(collected_events.clone(), parsed_events.clone()) {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to start trace: {:?}", e);
            return (Vec::new(), Vec::new());
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
    let raw_events = collected_events.lock().unwrap().clone();
    let parsed = parsed_events.lock().unwrap().clone();
    (raw_events, parsed)
}

/// Display a comparison of parsed events across configurations
/// This helps identify differences between flag combinations (e.g., PERF_FLT_FASTIO vs PERF_FLT_IO)
fn display_parsed_comparison(
    parsed_results: &Arc<Mutex<HashMap<String, Vec<ParsedFileIoEvent>>>>,
) {
    let results = parsed_results.lock().unwrap();

    log::info!("");
    log::info!("=== PARSED EVENT COMPARISON ===");
    log::info!("Comparing parsed event data across flag combinations to identify differences.");
    log::info!("");

    // Group events by (opcode, version) across all configs
    let mut events_by_type: HashMap<(u8, u8), HashMap<String, Vec<&ParsedFileIoEvent>>> =
        HashMap::new();

    for (config_name, events) in results.iter() {
        for event in events {
            events_by_type
                .entry((event.opcode, event.version))
                .or_default()
                .entry(config_name.clone())
                .or_default()
                .push(event);
        }
    }

    // Sort by opcode and version
    let mut sorted_types: Vec<_> = events_by_type.into_iter().collect();
    sorted_types.sort_by_key(|((op, ver), _)| (*op, *ver));

    for ((opcode, version), config_events) in sorted_types {
        let label = if let Some(known) = EVENT_REGISTRY.get(&(opcode, version)) {
            format!(
                "{} [{}] (Opcode={}, Version={})",
                known.event_name, known.class_name, opcode, version
            )
        } else {
            format!("UNKNOWN (Opcode={}, Version={})", opcode, version)
        };

        log::info!("{}", label);

        // Collect all unique configs that produced this event type
        let mut configs: Vec<_> = config_events.keys().cloned().collect();
        configs.sort();

        for config_name in &configs {
            let events = config_events.get(config_name).unwrap();
            log::info!("  {}: {} events", config_name, events.len());

            // Show a sample of the parsed data (first 3 events)
            for (i, event) in events.iter().take(3).enumerate() {
                log::info!(
                    "    [{}] PID={} TID={} data={:?}",
                    i + 1,
                    event.process_id,
                    event.thread_id,
                    event.event
                );
            }
            if events.len() > 3 {
                log::info!("    ... and {} more", events.len() - 3);
            }
        }

        // Highlight if this event type appears in some configs but not others
        if configs.len() < results.len() {
            let missing: Vec<_> = results
                .keys()
                .filter(|k| !configs.contains(k))
                .cloned()
                .collect();
            if !missing.is_empty() {
                log::info!("  NOT present in: {}", missing.join(", "));
            }
        }
    }

    // Special comparison for FltIoCompletion events (the main interest)
    log::info!("");
    log::info!("=== FLT IO COMPLETION COMPARISON ===");
    log::info!("Comparing FltIoCompletion events between PERF_FLT_FASTIO and PERF_FLT_IO");
    log::info!("");

    let flt_configs = ["GM:PERF_FLT_FASTIO", "GM:PERF_FLT_IO"];
    let flt_opcodes = [98u8, 99u8]; // PreOpCompletion, PostOpCompletion

    for &opcode in &flt_opcodes {
        let label = if let Some(known) = EVENT_REGISTRY.get(&(opcode, 3)) {
            known.event_name
        } else {
            "UNKNOWN"
        };

        log::info!("{} (Opcode={}):", label, opcode);

        let mut all_data: HashMap<String, Vec<String>> = HashMap::new();

        for &config_name in &flt_configs {
            if let Some(events) = results.get(config_name) {
                let flt_events: Vec<_> = events
                    .iter()
                    .filter(|e| e.opcode == opcode)
                    .collect();

                log::info!("  {}: {} events", config_name, flt_events.len());

                for event in &flt_events {
                    let data_str = format!("{:?}", event.event);
                    all_data
                        .entry(config_name.to_string())
                        .or_default()
                        .push(data_str);
                }
            } else {
                log::info!("  {}: no data", config_name);
            }
        }

        // Show unique data values for each config
        for &config_name in &flt_configs {
            if let Some(data) = all_data.get(config_name) {
                let unique: std::collections::HashSet<_> = data.iter().collect();
                log::info!(
                    "  {} unique data values: {}",
                    config_name,
                    unique.len()
                );
                for (i, d) in unique.iter().take(5).enumerate() {
                    log::info!("    [{}] {}", i + 1, d);
                }
                if unique.len() > 5 {
                    log::info!("    ... and {} more unique values", unique.len() - 5);
                }
            }
        }
    }
}
