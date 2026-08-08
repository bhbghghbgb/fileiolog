mod events;
mod file_ops;
mod fileio_events;
mod trace_session;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use events::{EVENT_REGISTRY, ParsedFileIoEvent};
use trace_session::{KernelTraceSession, TraceConfig};

const OUTPUT_DIR: &str = "flag_discovery_output";
const RESULTS_FILE: &str = "flag_discovery_results.json";
const RUNS_PER_COMBO: usize = 3;

/// A single trace flag — either set via EnableFlags or PERFINFO_GROUPMASK
struct Flag {
    name: String,
    enable_flags: Option<u32>,
    group_mask: Option<[u32; 8]>,
}

/// Tracks the minimal combination(s) found for each event type
struct EventDiscovery {
    /// The smallest combination size found so far
    best_size: usize,
    /// All combinations of that size that enable this event
    combinations: Vec<Vec<usize>>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== FileIo Flag Discovery ===");
    log::info!("Automatically discovering which flag(s) enable each FileIo event type.");
    log::info!("Runs per combination: {}", RUNS_PER_COMBO);
    log::info!("");

    let flags = build_all_flags();
    log::info!("Flags to test ({} total):", flags.len());
    for (i, f) in flags.iter().enumerate() {
        log::info!(
            "  [{}] {}: EF={:?} GM={:?}",
            i,
            f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)),
            f.group_mask.map(|m| format!("0x{:08X}", m.iter().fold(0u32, |a, &b| a | b)))
        );
    }
    log::info!("");

    let output_dir = Path::new(OUTPUT_DIR);
    if let Err(e) = fs::create_dir_all(output_dir) {
        log::warn!("Failed to create output directory: {}", e);
    }

    // Run discovery
    let discovery = discover(&flags);

    // Display and save results
    display_results(&discovery, &flags);
    save_results(&discovery, &flags, output_dir);
}

/// Build all 12 known flags
fn build_all_flags() -> Vec<Flag> {
    let mut flags = Vec::new();

    // ── EnableFlags-based flags ──────────────────────────────
    flags.push(Flag {
        name: "EF:DISK_FILE_IO".into(),
        enable_flags: Some(0x00000200),
        group_mask: None,
    });
    flags.push(Flag {
        name: "EF:FILE_IO".into(),
        enable_flags: Some(0x02000000),
        group_mask: None,
    });
    flags.push(Flag {
        name: "EF:FILE_IO_INIT".into(),
        enable_flags: Some(0x04000000),
        group_mask: None,
    });
    flags.push(Flag {
        name: "EF:VAMAP".into(),
        enable_flags: Some(0x00008000),
        group_mask: None,
    });

    // ── GroupMask-based flags (same values as above) ────────
    flags.push(Flag {
        name: "GM:PERF_FILENAME".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x00000200)),
    });
    flags.push(Flag {
        name: "GM:PERF_FILE_IO".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x02000000)),
    });
    flags.push(Flag {
        name: "GM:PERF_FILE_IO_INIT".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x04000000)),
    });
    flags.push(Flag {
        name: "GM:PERF_VAMAP".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x00008000)),
    });

    // ── Extended GroupMask flags (no EnableFlags equivalent) ─
    flags.push(Flag {
        name: "GM:PERF_FLT_IO_INIT".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80080000)),
    });
    flags.push(Flag {
        name: "GM:PERF_FLT_IO".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80100000)),
    });
    flags.push(Flag {
        name: "GM:PERF_FLT_FASTIO".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80200000)),
    });
    flags.push(Flag {
        name: "GM:PERF_FLT_IO_FAILURE".into(),
        enable_flags: None,
        group_mask: Some(build_group_mask(0x80400000)),
    });

    flags
}

/// Build a PERFINFO_GROUPMASK from a combined mask value
fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    let group_index = ((mask_value >> 29) & 0x07) as usize;
    masks[group_index] = mask_value;
    masks
}

/// Merge a set of flags into a single TraceConfig
fn merge_flags(flags: &[Flag], indices: &[usize]) -> TraceConfig {
    let mut enable_flags = 0u32;
    let mut group_mask = [0u32; 8];

    for &idx in indices {
        if let Some(ef) = flags[idx].enable_flags {
            enable_flags |= ef;
        }
        if let Some(gm) = flags[idx].group_mask {
            for i in 0..8 {
                group_mask[i] |= gm[i];
            }
        }
    }

    let session_name = format!(
        "FlagDiscovery-{}",
        indices
            .iter()
            .map(|i| flags[*i].name.as_str().replace(":", "_").replace("+", "_"))
            .collect::<Vec<_>>()
            .join("__")
    );

    TraceConfig {
        session_name,
        enable_flags: if enable_flags != 0 {
            Some(enable_flags)
        } else {
            None
        },
        group_mask: if group_mask.iter().any(|&x| x != 0) {
            Some(group_mask)
        } else {
            None
        },
    }
}

/// Run the main discovery algorithm
fn discover(flags: &[Flag]) -> HashMap<(u8, u8), EventDiscovery> {
    let mut discovered: HashMap<(u8, u8), EventDiscovery> = HashMap::new();
    let total_events = EVENT_REGISTRY.len();

    for size in 1..=flags.len() {
        let combos = generate_combinations(flags.len(), size);
        log::info!("========================================");
        log::info!(
            "Phase {}: Testing {} combinations of {} flags",
            size,
            combos.len(),
            size
        );
        log::info!("========================================");

        let mut new_events_this_level: Vec<String> = Vec::new();

        for (ci, combo_indices) in combos.iter().enumerate() {
            let combo_name: String = combo_indices
                .iter()
                .map(|i| flags[*i].name.as_str())
                .collect::<Vec<_>>()
                .join(" + ");

            log::info!(
                "  [{}/{}] {}",
                ci + 1,
                combos.len(),
                combo_name
            );

            // Test this combination multiple times
            let mut seen_events: HashSet<(u8, u8)> = HashSet::new();
            let config = merge_flags(flags, combo_indices);

            for run in 0..RUNS_PER_COMBO {
                log::info!("    Run {}/{}", run + 1, RUNS_PER_COMBO);
                let raw_events = run_single_test(&config);
                for event in &raw_events {
                    seen_events.insert((event.opcode, event.version));
                }
            }

            // Record newly discovered events
            let mut combo_new_events: Vec<String> = Vec::new();
            for event_key in &seen_events {
                if !EVENT_REGISTRY.contains_key(event_key) {
                    continue;
                }

                let entry = discovered
                    .entry(*event_key)
                    .or_insert_with(|| EventDiscovery {
                        best_size: usize::MAX,
                        combinations: Vec::new(),
                    });

                if size < entry.best_size {
                    // Found a smaller combination — replace
                    entry.best_size = size;
                    entry.combinations = vec![combo_indices.clone()];
                    let def = &EVENT_REGISTRY[event_key];
                    combo_new_events.push(format!(
                        "{}({}) [{}]",
                        def.event_name, def.version, def.class_name
                    ));
                } else if size == entry.best_size {
                    // Found another combination of same size — add
                    entry.combinations.push(combo_indices.clone());
                }
            }

            if !combo_new_events.is_empty() {
                new_events_this_level.extend(combo_new_events);
            }

            // Brief pause between tests
            if ci < combos.len() - 1 {
                std::thread::sleep(Duration::from_secs(1));
            }
        }

        if !new_events_this_level.is_empty() {
            log::info!("  New events discovered at size {}:", size);
            for name in &new_events_this_level {
                log::info!("    {}", name);
            }
        }

        let discovered_count = discovered
            .values()
            .filter(|d| d.best_size <= size)
            .count();
        log::info!(
            "Phase {} complete. Discovered: {}/{} event types.",
            size,
            discovered_count,
            total_events
        );

        // Early termination: all events discovered
        if discovered_count >= total_events {
            log::info!("All event types discovered! Stopping early.");
            break;
        }

        // Pause between phases
        log::info!("");
        std::thread::sleep(Duration::from_secs(2));
    }

    discovered
}

/// Generate all combinations of k indices from 0..n
fn generate_combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if k > n {
        return vec![];
    }
    let mut result = Vec::new();
    let mut combo = Vec::new();
    gen_combos_recursive(n, k, 0, &mut combo, &mut result);
    result
}

fn gen_combos_recursive(
    n: usize,
    k: usize,
    start: usize,
    combo: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if combo.len() == k {
        result.push(combo.clone());
        return;
    }
    for i in start..n {
        combo.push(i);
        gen_combos_recursive(n, k, i + 1, combo, result);
        combo.pop();
    }
}

/// Run a single test with the given config. Returns raw events.
fn run_single_test(config: &TraceConfig) -> Vec<events::FileIoRawEvent> {
    let collected_events: Arc<Mutex<Vec<events::FileIoRawEvent>>> =
        Arc::new(Mutex::new(Vec::new()));
    let parsed_events: Arc<Mutex<Vec<ParsedFileIoEvent>>> = Arc::new(Mutex::new(Vec::new()));

    let mut session = match KernelTraceSession::new(TraceConfig {
        session_name: config.session_name.clone(),
        enable_flags: config.enable_flags,
        group_mask: config.group_mask,
    }) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create trace session: {:?}", e);
            return Vec::new();
        }
    };

    let _trace_handle = match session.start(collected_events.clone(), parsed_events.clone()) {
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

    // Wait for trace to stabilize
    std::thread::sleep(Duration::from_millis(500));

    // Trigger file system events
    file_ops::trigger_all_file_operations();

    // Wait for events to arrive
    std::thread::sleep(Duration::from_secs(5));

    // Request rundown
    let _ = session.request_rundown();
    std::thread::sleep(Duration::from_secs(2));

    // Stop trace
    let _ = session.stop();
    let _ = processing_thread.join();

    let raw_events = collected_events.lock().unwrap().clone();
    raw_events
}

/// Display the discovery results
fn display_results(discovered: &HashMap<(u8, u8), EventDiscovery>, flags: &[Flag]) {
    log::info!("");
    log::info!("╔══════════════════════════════════════════════════════╗");
    log::info!("║           FLAG DISCOVERY RESULTS                    ║");
    log::info!("╚══════════════════════════════════════════════════════╝");

    // Group discovered events by their best_size
    let mut by_size: HashMap<usize, Vec<((u8, u8), &EventDiscovery)>> = HashMap::new();
    for (event_key, disc) in discovered {
        by_size
            .entry(disc.best_size)
            .or_default()
            .push((*event_key, disc));
    }

    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    // Track all event types covered so far (for showing "new only" under combos)
    let mut covered_events: HashSet<(u8, u8)> = HashSet::new();

    for size in &sizes {
        let events_at_size = &by_size[size];
        log::info!("");
        log::info!("--- Size {}: {} flag(s) ---", size, size);

        // Group by combination
        let mut combos_map: HashMap<Vec<usize>, Vec<(u8, u8)>> = HashMap::new();
        for (event_key, disc) in events_at_size {
            for combo in &disc.combinations {
                combos_map
                    .entry(combo.clone())
                    .or_default()
                    .push(*event_key);
            }
        }

        let mut sorted_combos: Vec<_> = combos_map.into_iter().collect();
        sorted_combos.sort_by(|a, b| {
            a.0.iter()
                .map(|i| flags[*i].name.as_str())
                .collect::<Vec<_>>()
                .join("+")
                .cmp(
                    &b.0
                        .iter()
                        .map(|i| flags[*i].name.as_str())
                        .collect::<Vec<_>>()
                        .join("+"),
                )
        });

        for (combo_indices, mut event_keys) in sorted_combos {
            event_keys.sort_by_key(|ek| {
                let def = &EVENT_REGISTRY[ek];
                (def.class_name, def.event_name, def.version)
            });

            let combo_name: String = combo_indices
                .iter()
                .map(|i| flags[*i].name.as_str())
                .collect::<Vec<_>>()
                .join(" + ");

            log::info!("  {}:", combo_name);

            // Filter: only show events not already covered by smaller combinations
            let new_events: Vec<_> = event_keys
                .iter()
                .filter(|ek| !covered_events.contains(ek))
                .collect();

            if new_events.is_empty() {
                log::info!("    (no new events — all covered by smaller combinations)");
            } else {
                for ek in &new_events {
                    let def = &EVENT_REGISTRY[ek];
                    log::info!(
                        "    {} [{}] (Opcode={}, Version={})",
                        def.event_name,
                        def.class_name,
                        def.opcode,
                        def.version
                    );
                }
            }
        }

        // Update covered events
        for (event_key, _) in events_at_size {
            covered_events.insert(*event_key);
        }
    }

    // Show undiscovered events
    let all_event_keys: HashSet<(u8, u8)> = EVENT_REGISTRY.keys().copied().collect();
    let undiscovered: Vec<_> = all_event_keys
        .difference(&covered_events)
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if !undiscovered.is_empty() {
        let mut sorted_undiscovered = undiscovered.clone();
        sorted_undiscovered.sort_by_key(|ek| {
            let def = &EVENT_REGISTRY[ek];
            (def.class_name, def.event_name, def.version)
        });

        log::info!("");
        log::warn!("--- Undiscovered Events ---");
        log::warn!(
            "The following {} event(s) were never observed:",
            sorted_undiscovered.len()
        );
        for ek in sorted_undiscovered {
            let def = &EVENT_REGISTRY[ek];
            log::warn!(
                "  {} [{}] (Opcode={}, Version={})",
                def.event_name,
                def.class_name,
                def.opcode,
                def.version
            );
        }
    }
}

/// Save results to JSON and text files
fn save_results(
    discovered: &HashMap<(u8, u8), EventDiscovery>,
    flags: &[Flag],
    output_dir: &Path,
) {
    // Build JSON-serializable output
    #[derive(serde::Serialize)]
    struct JsonResult {
        flags: Vec<FlagInfo>,
        discovered: Vec<EventResult>,
        undiscovered: Vec<EventInfo>,
    }

    #[derive(serde::Serialize)]
    struct FlagInfo {
        index: usize,
        name: String,
        enable_flags: Option<String>,
        group_mask: Option<String>,
    }

    #[derive(serde::Serialize, Clone)]
    struct EventResult {
        opcode: u8,
        version: u8,
        event_name: String,
        class_name: String,
        best_size: usize,
        combinations: Vec<Vec<String>>,
    }

    #[derive(serde::Serialize, Clone)]
    struct EventInfo {
        opcode: u8,
        version: u8,
        event_name: String,
        class_name: String,
    }

    let flag_infos: Vec<FlagInfo> = flags
        .iter()
        .enumerate()
        .map(|(i, f)| FlagInfo {
            index: i,
            name: f.name.clone(),
            enable_flags: f.enable_flags.map(|v| format!("0x{:08X}", v)),
            group_mask: f.group_mask.map(|m| {
                format!(
                    "[{:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}, {:08X}]",
                    m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7]
                )
            }),
        })
        .collect();

    let mut event_results: Vec<EventResult> = discovered
        .iter()
        .map(|(ek, disc)| {
            let def = &EVENT_REGISTRY[ek];
            EventResult {
                opcode: ek.0,
                version: ek.1,
                event_name: def.event_name.to_string(),
                class_name: def.class_name.to_string(),
                best_size: disc.best_size,
                combinations: disc
                    .combinations
                    .iter()
                    .map(|combo| {
                        combo
                            .iter()
                            .map(|i| flags[*i].name.clone())
                            .collect()
                    })
                    .collect(),
            }
        })
        .collect();
    event_results.sort_by_key(|e| (e.opcode, e.version));

    let all_event_keys: HashSet<(u8, u8)> = EVENT_REGISTRY.keys().copied().collect();
    let covered: HashSet<(u8, u8)> = discovered.keys().copied().collect();
    let mut undiscovered: Vec<EventInfo> = all_event_keys
        .difference(&covered)
        .map(|ek| {
            let def = &EVENT_REGISTRY[ek];
            EventInfo {
                opcode: ek.0,
                version: ek.1,
                event_name: def.event_name.to_string(),
                class_name: def.class_name.to_string(),
            }
        })
        .collect();
    undiscovered.sort_by_key(|e| (e.opcode, e.version));

    // Save JSON
    let result = JsonResult {
        flags: flag_infos,
        discovered: event_results.clone(),
        undiscovered: undiscovered.clone(),
    };
    let json_path = output_dir.join(RESULTS_FILE);
    match serde_json::to_string_pretty(&result) {
        Ok(json) => {
            if let Err(e) = fs::write(&json_path, &json) {
                log::error!("Failed to write {}: {}", json_path.display(), e);
            } else {
                log::info!("Results saved to {}", json_path.display());
            }
        }
        Err(e) => {
            log::error!("Failed to serialize results: {}", e);
        }
    }

    // Save human-readable text
    let txt_path = output_dir.join("flag_discovery_results.txt");
    let mut txt = String::new();
    txt.push_str("=== FLAG DISCOVERY RESULTS ===\n\n");

    txt.push_str("Flags tested:\n");
    for (i, f) in flags.iter().enumerate() {
        txt.push_str(&format!(
            "  [{}] {} (EF={:?}, GM={:?})\n",
            i,
            f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)),
            f.group_mask
        ));
    }
    txt.push('\n');

    // Group by best_size
    let mut by_size: HashMap<usize, Vec<&EventResult>> = HashMap::new();
    for er in &event_results {
        by_size.entry(er.best_size).or_default().push(er);
    }
    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    for size in &sizes {
        txt.push_str(&format!("--- Size {} ---\n", size));
        for er in &by_size[size] {
            for combo in &er.combinations {
                txt.push_str(&format!("  {} -> ", combo.join(" + ")));
                txt.push_str(&format!(
                    "{}({}) [{}]\n",
                    er.event_name, er.version, er.class_name
                ));
            }
        }
        txt.push('\n');
    }

    if !undiscovered.is_empty() {
        txt.push_str("--- Undiscovered Events ---\n");
        for e in &undiscovered {
            txt.push_str(&format!(
                "  {}({}) [{}]\n",
                e.event_name, e.version, e.class_name
            ));
        }
    }

    if let Err(e) = fs::write(&txt_path, &txt) {
        log::error!("Failed to write {}: {}", txt_path.display(), e);
    } else {
        log::info!("Results saved to {}", txt_path.display());
    }
}
