use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::flags::{self, Flag};
use crate::event_types::EventTypeInfo;
use crate::discovery::EventDiscovery;
use crate::RESULTS_FILE;

#[derive(serde::Serialize)]
struct JsonResult {
    flags: Vec<FlagInfo>,
    discovered: Vec<EventResult>,
    undiscovered: Vec<EventInfo>,
}

#[derive(serde::Serialize)]
struct FlagInfo { index: usize, name: String, enable_flags: Option<String>, group_mask: Option<String> }

#[derive(serde::Serialize, Clone)]
struct EventResult {
    opcode: u8, event_name: String, class_name: String,
    best_size: usize, combinations: Vec<Vec<String>>, observed_versions: Vec<u8>,
}

#[derive(serde::Serialize, Clone)]
struct EventInfo { opcode: u8, event_name: String, class_name: String }

// ── Progressive file logging ──────────────────────────────────────

pub(crate) fn write_combo_progress(
    dir: &Path, phase: usize, ci: usize,
    flags: &[Flag], discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
) {
    let path = dir.join(format!("phase_{}_combo_{}.json", phase, ci));
    let json = build_json(flags, discovered, event_types);
    if let Ok(s) = serde_json::to_string_pretty(&json) { let _ = fs::write(&path, s); }
}

pub(crate) fn cleanup_combo_files(dir: &Path, phase: usize) {
    let prefix = format!("phase_{}_combo_", phase);
    delete_matching(dir, &|name| name.starts_with(&prefix));
}

pub(crate) fn write_phase_summary(
    dir: &Path, phase: usize,
    flags: &[Flag], discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
) {
    let path = dir.join(format!("phase_{}.json", phase));
    let json = build_json(flags, discovered, event_types);
    if let Ok(s) = serde_json::to_string_pretty(&json) {
        if let Err(e) = fs::write(&path, &s) { log::warn!("Failed to write phase summary: {}", e); }
        else { log::info!("Phase {} summary saved to {}", phase, path.display()); }
    }
}

fn cleanup_phase_files(dir: &Path) {
    delete_matching(dir, &|name| name.starts_with("phase_") && name.ends_with(".json"));
}

fn delete_matching(dir: &Path, pred: &dyn Fn(&str) -> bool) {
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if pred(&name) { let _ = fs::remove_file(e.path()); }
        }
    }
}

// ── JSON building ─────────────────────────────────────────────────

fn build_json(
    flags: &[Flag], discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
) -> JsonResult {
    let flag_infos: Vec<FlagInfo> = flags.iter().enumerate().map(|(i, f)| FlagInfo {
        index: i, name: f.name.clone(),
        enable_flags: f.enable_flags.map(|v| format!("0x{:08X}", v)),
        group_mask: f.group_mask.map(|v| format!("0x{:08X}", v)),
    }).collect();

    let mut event_results: Vec<EventResult> = discovered.iter().map(|(opcode, disc)| {
        let et = event_types.iter().find(|et| et.opcode == *opcode);
        EventResult {
            opcode: *opcode,
            event_name: et.map(|e| e.event_name.to_string()).unwrap_or_default(),
            class_name: et.map(|e| e.class_name.to_string()).unwrap_or_default(),
            best_size: disc.best_size,
            combinations: disc.combinations.iter()
                .map(|c| c.iter().map(|i| flags[*i].name.clone()).collect()).collect(),
            observed_versions: disc.observed_versions.iter().copied().collect(),
        }
    }).collect();
    event_results.sort_by_key(|e| e.opcode);

    let found: HashSet<u8> = discovered.keys().copied().collect();
    let mut undiscovered: Vec<EventInfo> = event_types.iter()
        .filter(|et| !found.contains(&et.opcode))
        .map(|et| EventInfo { opcode: et.opcode, event_name: et.event_name.into(), class_name: et.class_name.into() })
        .collect();
    undiscovered.sort_by_key(|e| e.opcode);

    JsonResult { flags: flag_infos, discovered: event_results, undiscovered }
}

// ── Display ───────────────────────────────────────────────────────

pub(crate) fn display(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag],
    event_types: &[EventTypeInfo],
) {
    log::info!("");
    log::info!("=== FLAG DISCOVERY RESULTS ===");

    let mut by_size: HashMap<usize, Vec<(u8, &EventDiscovery)>> = HashMap::new();
    for (op, disc) in discovered { by_size.entry(disc.best_size).or_default().push((*op, disc)); }

    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    let mut covered: HashSet<u8> = HashSet::new();

    for size in &sizes {
        log::info!("");
        log::info!("--- Size {} ---", size);

        let mut combos_map: HashMap<Vec<usize>, Vec<u8>> = HashMap::new();
        for (op, disc) in &by_size[size] {
            for c in &disc.combinations { combos_map.entry(c.clone()).or_default().push(*op); }
        }

        let mut sorted: Vec<_> = combos_map.into_iter().collect();
        sorted.sort_by(|a, b| combo_sort_key(flags, &a.0).cmp(&combo_sort_key(flags, &b.0)));

        for (indices, mut ops) in sorted {
            ops.sort();
            let label = flags::combo_name(flags, &indices);
            let new_ops: Vec<u8> = ops.iter().filter(|op| !covered.contains(op)).copied().collect();

            log::info!("  {}:", label);
            if new_ops.is_empty() {
                log::info!("    (no new events — all covered by smaller combinations)");
            } else {
                for op in &new_ops {
                    let et = event_types.iter().find(|et| et.opcode == *op);
                    let vers = observed_versions_str(discovered.get(op));
                    if let Some(et) = et {
                        log::info!("    {} [{}] (Opcode={}, Versions={{{}}})", et.event_name, et.class_name, op, vers);
                    }
                }
            }
        }
        for (op, _) in &by_size[size] { covered.insert(*op); }
    }

    let all: HashSet<u8> = event_types.iter().map(|et| et.opcode).collect();
    let missing: Vec<_> = all.difference(&covered).copied().collect::<Vec<_>>().into_iter().collect();
    if !missing.is_empty() {
        let mut sorted = missing; sorted.sort();
        log::info!("");
        log::warn!("--- Undiscovered Events ---");
        log::warn!("The following {} event type(s) were never observed:", sorted.len());
        for op in sorted {
            if let Some(et) = event_types.iter().find(|et| et.opcode == op) {
                log::warn!("  {} [{}] (Opcode={})", et.event_name, et.class_name, op);
            }
        }
    }
}

fn combo_sort_key(flags: &[Flag], indices: &[usize]) -> String {
    indices.iter().map(|i| flags[*i].name.as_str()).collect::<Vec<_>>().join("+")
}

fn observed_versions_str(disc: Option<&EventDiscovery>) -> String {
    disc.map(|d| {
        let mut v: Vec<u8> = d.observed_versions.iter().copied().collect();
        v.sort();
        v.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    }).unwrap_or_default()
}

// ── Final save ────────────────────────────────────────────────────

pub(crate) fn save_final(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag],
    event_types: &[EventTypeInfo],
    dir: &Path,
) {
    cleanup_phase_files(dir);
    let result = build_json(flags, discovered, event_types);

    // JSON
    let json_path = dir.join(RESULTS_FILE);
    match serde_json::to_string_pretty(&result) {
        Ok(json) => match fs::write(&json_path, &json) {
            Ok(_) => log::info!("Final results saved to {}", json_path.display()),
            Err(e) => log::error!("Failed to write {}: {}", json_path.display(), e),
        },
        Err(e) => log::error!("Failed to serialize results: {}", e),
    }

    // Text
    let txt_path = dir.join("flag_discovery_results.txt");
    let mut txt = String::from("=== FLAG DISCOVERY RESULTS ===\n\n");

    txt.push_str("Flags tested:\n");
    for (i, f) in flags.iter().enumerate() {
        txt.push_str(&format!("  [{}] {} (EF={:?}, GM={:?})\n", i, f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)), f.group_mask.map(|v| format!("0x{:08X}", v))));
    }
    txt.push('\n');

    let mut by_size: HashMap<usize, Vec<&EventResult>> = HashMap::new();
    for er in &result.discovered { by_size.entry(er.best_size).or_default().push(er); }
    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    for size in &sizes {
        txt.push_str(&format!("--- Size {} ---\n", size));
        for er in &by_size[size] {
            for c in &er.combinations {
                txt.push_str(&format!("  {} -> {} [{}] (Opcode={}, V={{{}}})\n",
                    c.join(" + "), er.event_name, er.class_name, er.opcode,
                    er.observed_versions.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")));
            }
        }
        txt.push('\n');
    }

    if !result.undiscovered.is_empty() {
        txt.push_str("--- Undiscovered Events ---\n");
        for e in &result.undiscovered {
            txt.push_str(&format!("  {} [{}] (Opcode={})\n", e.event_name, e.class_name, e.opcode));
        }
    }

    match fs::write(&txt_path, &txt) {
        Ok(_) => log::info!("Final results saved to {}", txt_path.display()),
        Err(e) => log::error!("Failed to write {}: {}", txt_path.display(), e),
    }
}
