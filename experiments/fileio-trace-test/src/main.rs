mod events;
mod file_ops;
mod fileio_events;
mod persist;
mod trace_session;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clap::Parser as ClapParser;
use events::{EVENT_REGISTRY, ParsedFileIoEvent};
use trace_session::{KernelTraceSession, TraceConfig};

#[derive(Debug, ClapParser)]
#[command(name = "fileio-trace-test")]
#[command(about = "Test FileIo ETW trace configurations")]
struct Args {
    /// Output directory for results
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

const EVENTS_DIR: &str = "fileio_events";

/// Test configuration for a single flag/mask combination
struct TestConfig {
    name: String,
    enable_flags: Option<u32>,
    group_mask: Option<[u32; 8]>,
}

fn main() {
    let args = Args::parse();
    let output_dir = &args.output;
    let persist_path = output_dir.join("fileio_test_results.json");
    let events_dir = output_dir.join(EVENTS_DIR);

    let _ = fs::create_dir_all(output_dir);
    fileiolog::logging::init_logging(output_dir, "fileio-trace-test");

    log::info!("=== FileIo ETW Trace Test ===");
    log::info!("This test will iterate through different EnableFlags and PERFINFO_GROUPMASK");
    log::info!("configurations to discover which FileIo event types are enabled by each.");
    log::info!("");

    // Load previous persisted results
    let mut persisted = persist::load(&persist_path);

    // Create events output directory
    if let Err(e) = fs::create_dir_all(&events_dir) {
        log::warn!("Failed to create events directory {}: {}", events_dir.display(), e);
    }

    // Define all test configurations
    let test_configs = build_test_configs();

    log::info!("Total configurations to test: {}", test_configs.len());
    log::info!("");

    // Shared storage for raw events (needed for the original per-config display)
    let results: Arc<Mutex<HashMap<String, Vec<events::FileIoRawEvent>>>> =
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

        // Write parsed events to file immediately after session ends
        // (flush to disk before next session to avoid contributing to events)
        write_events_to_file(&events_dir, &config.name, &parsed_events);

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

        // Brief pause between tests
        if i < test_configs.len() - 1 {
            log::info!("  Pausing 2 seconds before next test...");
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    // Merge current run with persisted data and save
    persist::merge(&mut persisted, &current_counts);
    persist::save(&persist_path, &persisted);

    // Display cumulative results
    persist::display(&persisted, &current_counts);

    // Write human-readable text output
    write_text_summary(output_dir, &persisted, &current_counts, &test_configs);
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

/// Write parsed events for a single configuration to a file
fn write_events_to_file(
    events_dir: &Path,
    config_name: &str,
    events: &[ParsedFileIoEvent],
) {
    // Sanitize config name for filename
    let safe_name = config_name
        .replace(":", "_")
        .replace("+", "_")
        .replace(" ", "_");
    let file_path = events_dir.join(format!("{}.json", safe_name));

    log::info!(
        "  Writing {} events for '{}' to {}",
        events.len(),
        config_name,
        file_path.display()
    );

    match serde_json::to_string_pretty(events) {
        Ok(json) => {
            if let Err(e) = fs::write(&file_path, json) {
                log::error!("Failed to write {}: {}", file_path.display(), e);
            }
        }
        Err(e) => {
            log::error!("Failed to serialize events for '{}': {}", config_name, e);
        }
    }
}

/// Write a human-readable text summary of the cumulative results
fn write_text_summary(
    output_dir: &Path,
    persisted: &persist::PersistedData,
    current: &HashMap<String, HashMap<String, usize>>,
    test_configs: &[TestConfig],
) {
    use events::EVENT_REGISTRY;

    let txt_path = output_dir.join("fileio_test_results.txt");
    let mut txt = String::from("=== FileIo ETW Trace Test Results ===\n\n");
    txt.push_str(&format!("Total runs: {}\n\n", persisted.total_runs));

    txt.push_str("Configurations tested:\n");
    for config in test_configs {
        let ef = config.enable_flags.map(|f| format!("0x{:08X}", f)).unwrap_or_else(|| "None".into());
        let gm = config.group_mask.map(|m| {
            format!("[{:08X},{:08X},{:08X},{:08X},{:08X},{:08X},{:08X},{:08X}]",
                m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7])
        }).unwrap_or_else(|| "None".into());
        txt.push_str(&format!("  {}: EF={}, GM={}\n", config.name, ef, gm));
    }
    txt.push('\n');

    // Collect all event keys across all configs
    let mut all_events: Vec<(String, String, Vec<(String, usize, usize)>)> = Vec::new();
    let mut seen_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (_config_name, persisted_events) in &persisted.config_events {
        for (ek, &_cumulative_count) in persisted_events {
            if seen_keys.contains(ek) {
                continue;
            }
            seen_keys.insert(ek.clone());

            let mut config_entries: Vec<(String, usize, usize)> = Vec::new();
            for (cn, pe) in &persisted.config_events {
                if let Some(&cc) = pe.get(ek) {
                    let current_count = current.get(cn).and_then(|ce| ce.get(ek)).copied().unwrap_or(0);
                    config_entries.push((cn.clone(), current_count, cc));
                }
            }
            config_entries.sort_by(|a, b| a.0.cmp(&b.0));

            let (opcode, version) = match parse_event_key(ek) {
                Some(v) => v,
                None => continue,
            };
            let label = if let Some(known) = EVENT_REGISTRY.get(&(opcode, version)) {
                format!("{} [{}] (Opcode={}, Version={})", known.event_name, known.class_name, opcode, version)
            } else {
                format!("UNKNOWN (Opcode={}, Version={})", opcode, version)
            };
            all_events.push((label, ek.clone(), config_entries));
        }
    }

    // Sort by opcode/version
    all_events.sort_by(|a, b| {
        let (op_a, ver_a) = parse_event_key(&a.1).unwrap_or((0, 0));
        let (op_b, ver_b) = parse_event_key(&b.1).unwrap_or((0, 0));
        op_a.cmp(&op_b).then(ver_a.cmp(&ver_b))
    });

    txt.push_str("=== Cumulative Event Counts ===\n\n");
    for (label, _ek, config_entries) in &all_events {
        txt.push_str(&format!("{}\n", label));
        for (config_name, current_count, cumulative_count) in config_entries {
            if *current_count > 0 {
                txt.push_str(&format!("    {}: {} this run, {} cumulative\n", config_name, current_count, cumulative_count));
            } else {
                txt.push_str(&format!("    {}: 0 this run, {} cumulative\n", config_name, cumulative_count));
            }
        }
        txt.push('\n');
    }

    // Warn about never-received events
    let received_keys: std::collections::HashSet<String> = persisted.config_events.values().flat_map(|m| m.keys().cloned()).collect();
    let mut unreceived: Vec<_> = EVENT_REGISTRY.iter()
        .filter(|((op, ver), _)| !received_keys.contains(&format!("{}:{}", op, ver)))
        .collect();
    unreceived.sort_by_key(|((op, ver), _)| (*op, *ver));

    if !unreceived.is_empty() {
        txt.push_str("--- Events NEVER received ---\n");
        for ((opcode, version), def) in &unreceived {
            txt.push_str(&format!("  {} [{}] (Opcode={}, Version={})\n", def.event_name, def.class_name, opcode, version));
        }
    }

    if let Err(e) = fs::write(&txt_path, &txt) {
        log::error!("Failed to write text output: {}", e);
    } else {
        log::info!("Text output saved to {}", txt_path.display());
    }
}

fn parse_event_key(key: &str) -> Option<(u8, u8)> {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() != 2 { return None; }
    let opcode = parts[0].parse::<u8>().ok()?;
    let version = parts[1].parse::<u8>().ok()?;
    Some((opcode, version))
}
