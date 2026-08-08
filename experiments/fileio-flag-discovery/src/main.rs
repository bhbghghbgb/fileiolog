mod events;
mod file_ops;
mod fileio_events;
mod trace_session;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use events::{EVENT_REGISTRY, ParsedFileIoEvent};
use trace_session::{KernelTraceSession, TraceConfig};

const OUTPUT_DIR: &str = "flag_discovery_output";
const RESULTS_FILE: &str = "flag_discovery_results.json";
const RUNS_PER_COMBO: usize = 3;
const MAX_COMBO_SIZE: usize = 3;

/// A single trace flag — either set via EnableFlags or PERFINFO_GROUPMASK
struct Flag {
    name: String,
    enable_flags: Option<u32>,
    group_mask: Option<u32>,
}

/// Describes a unique event type keyed by opcode (version-agnostic)
struct EventTypeInfo {
    opcode: u8,
    event_name: &'static str,
    class_name: &'static str,
}

/// Tracks the minimal combination(s) found for each opcode
struct EventDiscovery {
    best_size: usize,
    combinations: Vec<Vec<usize>>,
    /// Which (opcode, version) pairs were actually observed (for reporting)
    observed_versions: HashSet<u8>,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("=== FileIo Flag Discovery ===");
    log::info!("Automatically discovering which flag(s) enable each FileIo event type.");
    log::info!("Runs per combination: {}", RUNS_PER_COMBO);
    log::info!("Max combination size: {}", MAX_COMBO_SIZE);
    log::info!("Event matching is version-agnostic (any version of an opcode counts).");
    log::info!("");

    let flags = build_all_flags();
    let event_types = build_event_types();

    log::info!("Flags to test ({} total):", flags.len());
    for (i, f) in flags.iter().enumerate() {
        log::info!(
            "  [{}] {}: EF={:?} GM={:?}",
            i,
            f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)),
            f.group_mask
        );
    }
    log::info!("");
    log::info!("Unique event types (opcodes) to discover: {}", event_types.len());
    for et in &event_types {
        log::info!("  Opcode={} {} [{}]", et.opcode, et.event_name, et.class_name);
    }
    log::info!("");

    let output_dir = Path::new(OUTPUT_DIR);
    if let Err(e) = fs::create_dir_all(output_dir) {
        log::warn!("Failed to create output directory: {}", e);
    }

    let discovery = discover(&flags, &event_types, output_dir);

    display_results(&discovery, &flags, &event_types);
    save_final_results(&discovery, &flags, &event_types, output_dir);
}

/// Build all 12 known flags
fn build_all_flags() -> Vec<Flag> {
    let mut flags = Vec::new();

    flags.push(Flag { name: "EF:DISK_FILE_IO".into(), enable_flags: Some(0x00000200), group_mask: None });
    flags.push(Flag { name: "EF:FILE_IO".into(), enable_flags: Some(0x02000000), group_mask: None });
    flags.push(Flag { name: "EF:FILE_IO_INIT".into(), enable_flags: Some(0x04000000), group_mask: None });
    flags.push(Flag { name: "EF:VAMAP".into(), enable_flags: Some(0x00008000), group_mask: None });

    flags.push(Flag { name: "GM:PERF_FILENAME".into(), enable_flags: None, group_mask: Some(0x00000200) });
    flags.push(Flag { name: "GM:PERF_FILE_IO".into(), enable_flags: None, group_mask: Some(0x02000000) });
    flags.push(Flag { name: "GM:PERF_FILE_IO_INIT".into(), enable_flags: None, group_mask: Some(0x04000000) });
    flags.push(Flag { name: "GM:PERF_VAMAP".into(), enable_flags: None, group_mask: Some(0x00008000) });

    flags.push(Flag { name: "GM:PERF_FLT_IO_INIT".into(), enable_flags: None, group_mask: Some(0x80080000) });
    flags.push(Flag { name: "GM:PERF_FLT_IO".into(), enable_flags: None, group_mask: Some(0x80100000) });
    flags.push(Flag { name: "GM:PERF_FLT_FASTIO".into(), enable_flags: None, group_mask: Some(0x80200000) });
    flags.push(Flag { name: "GM:PERF_FLT_IO_FAILURE".into(), enable_flags: None, group_mask: Some(0x80400000) });

    flags
}

/// Build unique event types from EVENT_REGISTRY, keyed by opcode.
/// For opcodes with multiple versions, pick the latest version's name.
fn build_event_types() -> Vec<EventTypeInfo> {
    let mut by_opcode: BTreeMap<u8, Vec<&events::FileIoEventDef>> = BTreeMap::new();
    for (key, def) in EVENT_REGISTRY.iter() {
        by_opcode.entry(key.0).or_default().push(def);
    }

    by_opcode
        .into_iter()
        .map(|(opcode, defs)| {
            // Pick the definition with the highest version as canonical
            let canonical = defs.iter().max_by_key(|d| d.version).unwrap();
            EventTypeInfo {
                opcode,
                event_name: canonical.event_name,
                class_name: canonical.class_name,
            }
        })
        .collect()
}

/// Merge a set of flags into a single TraceConfig
fn merge_flags(flags: &[Flag], indices: &[usize]) -> TraceConfig {
    let mut enable_flags = 0u32;
    let mut group_mask = 0u32;

    for &idx in indices {
        if let Some(ef) = flags[idx].enable_flags {
            enable_flags |= ef;
        }
        if let Some(gm) = flags[idx].group_mask {
            group_mask |= gm;
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
        enable_flags: if enable_flags != 0 { Some(enable_flags) } else { None },
        group_mask: if group_mask != 0 { Some(build_group_mask(group_mask)) } else { None },
    }
}

fn build_group_mask(mask_value: u32) -> [u32; 8] {
    let mut masks = [0u32; 8];
    let group_index = ((mask_value >> 29) & 0x07) as usize;
    masks[group_index] = mask_value;
    masks
}

/// Run the main discovery algorithm. Returns discoveries keyed by opcode.
fn discover(
    flags: &[Flag],
    event_types: &[EventTypeInfo],
    output_dir: &Path,
) -> HashMap<u8, EventDiscovery> {
    let mut discovered: HashMap<u8, EventDiscovery> = HashMap::new();
    let total_event_types = event_types.len();

    for size in 1..=MAX_COMBO_SIZE {
        let combos = generate_combinations(flags.len(), size);
        log::info!("========================================");
        log::info!(
            "Phase {}: Testing {} combinations of {} flags",
            size,
            combos.len(),
            size
        );
        log::info!("========================================");

        let mut new_events_this_phase: Vec<String> = Vec::new();

        for (ci, combo_indices) in combos.iter().enumerate() {
            let combo_name = combo_name_str(flags, combo_indices);

            log::info!("  [{}/{}] {}", ci + 1, combos.len(), combo_name);

            let mut seen_opcodes: HashSet<u8> = HashSet::new();
            let mut seen_versions: HashMap<u8, HashSet<u8>> = HashMap::new();
            let config = merge_flags(flags, combo_indices);

            for run in 0..RUNS_PER_COMBO {
                log::info!("    Run {}/{}", run + 1, RUNS_PER_COMBO);
                let raw_events = run_single_test(&config);
                for event in &raw_events {
                    seen_versions
                        .entry(event.opcode)
                        .or_default()
                        .insert(event.version);
                    seen_opcodes.insert(event.opcode);
                }
            }

            // Record newly discovered opcodes
            let mut combo_new_events: Vec<String> = Vec::new();
            for &opcode in &seen_opcodes {
                if !event_types.iter().any(|et| et.opcode == opcode) {
                    continue;
                }

                let entry = discovered
                    .entry(opcode)
                    .or_insert_with(|| EventDiscovery {
                        best_size: usize::MAX,
                        combinations: Vec::new(),
                        observed_versions: HashSet::new(),
                    });

                // Track which versions we saw
                if let Some(versions) = seen_versions.get(&opcode) {
                    entry.observed_versions.extend(versions);
                }

                if size < entry.best_size {
                    entry.best_size = size;
                    entry.combinations = vec![combo_indices.clone()];
                    if let Some(et) = event_types.iter().find(|et| et.opcode == opcode) {
                        combo_new_events.push(format!(
                            "{} [{}] (Opcode={})",
                            et.event_name, et.class_name, opcode
                        ));
                    }
                } else if size == entry.best_size {
                    entry.combinations.push(combo_indices.clone());
                }
            }

            if !combo_new_events.is_empty() {
                new_events_this_phase.extend(combo_new_events.iter().map(|s| s.clone()));
            }

            // Write per-combo progress file
            write_combo_progress(output_dir, size, ci, flags, &discovered, event_types);

            if ci < combos.len() - 1 {
                std::thread::sleep(Duration::from_secs(1));
            }
        }

        if !new_events_this_phase.is_empty() {
            log::info!("  New events discovered at size {}:", size);
            for name in &new_events_this_phase {
                log::info!("    {}", name);
            }
        }

        let discovered_count = discovered.values().filter(|d| d.best_size <= size).count();
        log::info!(
            "Phase {} complete. Discovered: {}/{} event types.",
            size,
            discovered_count,
            total_event_types
        );

        // Delete combo progress files and write phase summary
        cleanup_combo_files(output_dir, size);
        write_phase_summary(output_dir, size, flags, &discovered, event_types);

        // Early termination: all events discovered
        if discovered_count >= total_event_types {
            log::info!("All event types discovered! Stopping early.");
            break;
        }

        log::info!("");
        std::thread::sleep(Duration::from_secs(2));
    }

    discovered
}

fn combo_name_str(flags: &[Flag], indices: &[usize]) -> String {
    indices
        .iter()
        .map(|i| flags[*i].name.as_str())
        .collect::<Vec<_>>()
        .join(" + ")
}

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

    let process_handle = session.get_trace_handle();
    let processing_thread = std::thread::spawn(move || {
        use ferrisetw::trace::TraceTrait;
        let _ = <ferrisetw::trace::KernelTrace as TraceTrait>::process_from_handle(process_handle);
    });

    std::thread::sleep(Duration::from_millis(500));
    file_ops::trigger_all_file_operations();
    std::thread::sleep(Duration::from_secs(5));
    let _ = session.request_rundown();
    std::thread::sleep(Duration::from_secs(2));
    let _ = session.stop();
    let _ = processing_thread.join();

    collected_events.lock().unwrap().clone()
}

// ── Progressive file logging ──────────────────────────────────────

fn write_combo_progress(
    output_dir: &Path,
    phase: usize,
    combo_index: usize,
    flags: &[Flag],
    discovered: &HashMap<u8, EventDiscovery>,
    event_types: &[EventTypeInfo],
) {
    let path = output_dir.join(format!("phase_{}_combo_{}.json", phase, combo_index));
    let json = build_discovery_json(flags, discovered, event_types);
    if let Ok(json_str) = serde_json::to_string_pretty(&json) {
        let _ = fs::write(&path, json_str);
    }
}

fn cleanup_combo_files(output_dir: &Path, phase: usize) {
    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(&format!("phase_{}_combo_", phase)) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

fn write_phase_summary(
    output_dir: &Path,
    phase: usize,
    flags: &[Flag],
    discovered: &HashMap<u8, EventDiscovery>,
    event_types: &[EventTypeInfo],
) {
    let path = output_dir.join(format!("phase_{}.json", phase));
    let json = build_discovery_json(flags, discovered, event_types);
    if let Ok(json_str) = serde_json::to_string_pretty(&json) {
        if let Err(e) = fs::write(&path, &json_str) {
            log::warn!("Failed to write phase summary: {}", e);
        } else {
            log::info!("Phase {} summary saved to {}", phase, path.display());
        }
    }
}

fn cleanup_phase_files(output_dir: &Path) {
    if let Ok(entries) = fs::read_dir(output_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("phase_") && name_str.ends_with(".json") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

// ── JSON building (shared by progress, phase, and final) ──────────

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
    event_name: String,
    class_name: String,
    best_size: usize,
    combinations: Vec<Vec<String>>,
    observed_versions: Vec<u8>,
}

#[derive(serde::Serialize, Clone)]
struct EventInfo {
    opcode: u8,
    event_name: String,
    class_name: String,
}

fn build_discovery_json(
    flags: &[Flag],
    discovered: &HashMap<u8, EventDiscovery>,
    event_types: &[EventTypeInfo],
) -> JsonResult {
    let flag_infos: Vec<FlagInfo> = flags
        .iter()
        .enumerate()
        .map(|(i, f)| FlagInfo {
            index: i,
            name: f.name.clone(),
            enable_flags: f.enable_flags.map(|v| format!("0x{:08X}", v)),
            group_mask: f.group_mask.map(|v| format!("0x{:08X}", v)),
        })
        .collect();

    let mut event_results: Vec<EventResult> = discovered
        .iter()
        .map(|(opcode, disc)| {
            let et = event_types.iter().find(|et| et.opcode == *opcode);
            EventResult {
                opcode: *opcode,
                event_name: et.map(|e| e.event_name.to_string()).unwrap_or_default(),
                class_name: et.map(|e| e.class_name.to_string()).unwrap_or_default(),
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
                observed_versions: disc.observed_versions.iter().copied().collect(),
            }
        })
        .collect();
    event_results.sort_by_key(|e| e.opcode);

    let discovered_opcodes: HashSet<u8> = discovered.keys().copied().collect();
    let mut undiscovered: Vec<EventInfo> = event_types
        .iter()
        .filter(|et| !discovered_opcodes.contains(&et.opcode))
        .map(|et| EventInfo {
            opcode: et.opcode,
            event_name: et.event_name.to_string(),
            class_name: et.class_name.to_string(),
        })
        .collect();
    undiscovered.sort_by_key(|e| e.opcode);

    JsonResult {
        flags: flag_infos,
        discovered: event_results,
        undiscovered,
    }
}

// ── Display ───────────────────────────────────────────────────────

fn display_results(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag],
    event_types: &[EventTypeInfo],
) {
    log::info!("");
    log::info!("=== FLAG DISCOVERY RESULTS ===");

    let mut by_size: HashMap<usize, Vec<(u8, &EventDiscovery)>> = HashMap::new();
    for (opcode, disc) in discovered {
        by_size
            .entry(disc.best_size)
            .or_default()
            .push((*opcode, disc));
    }

    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    let mut covered_opcodes: HashSet<u8> = HashSet::new();

    for size in &sizes {
        let events_at_size = &by_size[size];
        log::info!("");
        log::info!("--- Size {} ---", size);

        // Group by combination
        let mut combos_map: HashMap<Vec<usize>, Vec<u8>> = HashMap::new();
        for (opcode, disc) in events_at_size {
            for combo in &disc.combinations {
                combos_map
                    .entry(combo.clone())
                    .or_default()
                    .push(*opcode);
            }
        }

        let mut sorted_combos: Vec<_> = combos_map.into_iter().collect();
        sorted_combos.sort_by(|a, b| {
            a.0.iter()
                .map(|i| flags[*i].name.as_str())
                .collect::<Vec<_>>()
                .join("+")
                .cmp(
                    &b.0.iter()
                        .map(|i| flags[*i].name.as_str())
                        .collect::<Vec<_>>()
                        .join("+"),
                )
        });

        for (combo_indices, mut opcodes) in sorted_combos {
            opcodes.sort();

            let combo_label = combo_name_str(flags, &combo_indices);

            // Only show opcodes not already covered by smaller combinations
            let new_opcodes: Vec<u8> = opcodes
                .iter()
                .filter(|op| !covered_opcodes.contains(op))
                .copied()
                .collect();

            if new_opcodes.is_empty() {
                log::info!("  {}:", combo_label);
                log::info!("    (no new events — all covered by smaller combinations)");
            } else {
                log::info!("  {}:", combo_label);
                for opcode in &new_opcodes {
                    let et = event_types.iter().find(|et| et.opcode == *opcode);
                    let disc = discovered.get(opcode);
                    let versions: String = disc
                        .map(|d| {
                            let mut v: Vec<u8> = d.observed_versions.iter().copied().collect();
                            v.sort();
                            v.iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        })
                        .unwrap_or_default();
                    if let Some(et) = et {
                        log::info!(
                            "    {} [{}] (Opcode={}, Versions={{{}}})",
                            et.event_name,
                            et.class_name,
                            opcode,
                            versions
                        );
                    }
                }
            }
        }

        for (opcode, _) in events_at_size {
            covered_opcodes.insert(*opcode);
        }
    }

    // Undiscovered
    let all_opcodes: HashSet<u8> = event_types.iter().map(|et| et.opcode).collect();
    let undiscovered: Vec<_> = all_opcodes.difference(&covered_opcodes).collect();
    if !undiscovered.is_empty() {
        let mut sorted = undiscovered.clone();
        sorted.sort();
        log::info!("");
        log::warn!("--- Undiscovered Events ---");
        log::warn!("The following {} event type(s) were never observed:", sorted.len());
        for opcode in sorted {
            let et = event_types.iter().find(|et| et.opcode == *opcode);
            if let Some(et) = et {
                log::warn!("  {} [{}] (Opcode={})", et.event_name, et.class_name, opcode);
            }
        }
    }
}

// ── Final save ────────────────────────────────────────────────────

fn save_final_results(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag],
    event_types: &[EventTypeInfo],
    output_dir: &Path,
) {
    // Clean up phase files
    cleanup_phase_files(output_dir);

    let result = build_discovery_json(flags, discovered, event_types);

    // Save JSON
    let json_path = output_dir.join(RESULTS_FILE);
    match serde_json::to_string_pretty(&result) {
        Ok(json) => {
            if let Err(e) = fs::write(&json_path, &json) {
                log::error!("Failed to write {}: {}", json_path.display(), e);
            } else {
                log::info!("Final results saved to {}", json_path.display());
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
            f.group_mask.map(|v| format!("0x{:08X}", v))
        ));
    }
    txt.push('\n');

    let mut by_size: HashMap<usize, Vec<&EventResult>> = HashMap::new();
    for er in &result.discovered {
        by_size.entry(er.best_size).or_default().push(er);
    }
    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    for size in &sizes {
        txt.push_str(&format!("--- Size {} ---\n", size));
        for er in &by_size[size] {
            for combo in &er.combinations {
                txt.push_str(&format!(
                    "  {} -> {} [{}] (Opcode={}, V={{{}}})\n",
                    combo.join(" + "),
                    er.event_name,
                    er.class_name,
                    er.opcode,
                    er.observed_versions
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
        }
        txt.push('\n');
    }

    if !result.undiscovered.is_empty() {
        txt.push_str("--- Undiscovered Events ---\n");
        for e in &result.undiscovered {
            txt.push_str(&format!(
                "  {} [{}] (Opcode={})\n",
                e.event_name, e.class_name, e.opcode
            ));
        }
    }

    if let Err(e) = fs::write(&txt_path, &txt) {
        log::error!("Failed to write {}: {}", txt_path.display(), e);
    } else {
        log::info!("Final results saved to {}", txt_path.display());
    }
}
