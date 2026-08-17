use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::discovery::{EventDiscovery, ObservedTrace};
use crate::event_types::EventTypeInfo;
use crate::events::{self, max_known_version};
use crate::flags::{self, Flag};
use crate::RESULTS_FILE;

// ── Warning structures ────────────────────────────────────────────

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "kind")]
enum WarningKind {
    #[serde(rename = "unknown_opcode")]
    UnknownOpcode,
    #[serde(rename = "higher_version")]
    HigherVersion { max_known: u8 },
}

#[derive(serde::Serialize, Clone)]
struct WarningEntry {
    opcode: u8,
    version: u8,
    count: u64,
    class_name: String,
    event_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_known_version: Option<u8>,
    first_combo: Vec<String>,
    #[serde(flatten)]
    kind: WarningKind,
}

fn build_warnings(
    observed: &ObservedTrace,
    flags: &[Flag],
) -> Vec<WarningEntry> {
    let mut entries: Vec<WarningEntry> = Vec::new();

    for (&(opcode, version), &count) in &observed.pairs {
        match max_known_version(opcode) {
            None => {
                // Unknown opcode entirely
                let canonical = events::canonical_def(opcode);
                let first = observed
                    .first_combo
                    .get(&(opcode, version))
                    .map(|c| c.iter().map(|i| flags[*i].name.clone()).collect())
                    .unwrap_or_default();

                entries.push(WarningEntry {
                    opcode,
                    version,
                    count,
                    class_name: canonical.map(|d| d.class_name.to_string()).unwrap_or_default(),
                    event_name: canonical.map(|d| d.event_name.to_string()).unwrap_or_default(),
                    max_known_version: None,
                    first_combo: first,
                    kind: WarningKind::UnknownOpcode,
                });
            }
            Some(max) => {
                if version > max {
                    let canonical = events::canonical_def(opcode).unwrap();
                    let first = observed
                        .first_combo
                        .get(&(opcode, version))
                        .map(|c| c.iter().map(|i| flags[*i].name.clone()).collect())
                        .unwrap_or_default();

                    entries.push(WarningEntry {
                        opcode,
                        version,
                        count,
                        class_name: canonical.class_name.to_string(),
                        event_name: canonical.event_name.to_string(),
                        max_known_version: Some(max),
                        first_combo: first,
                        kind: WarningKind::HigherVersion { max_known: max },
                    });
                }
                // version <= max → not a warning
            }
        }
    }

    // Sort by opcode then version for stable output
    entries.sort_by(|a, b| a.opcode.cmp(&b.opcode).then(a.version.cmp(&b.version)));
    entries
}

// ── JSON schemas ──────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct RunFile {
    r#type: &'static str,
    metadata: RunMeta,
    run_result: RunResult,
    cumulative: CumulativeState,
}

#[derive(serde::Serialize)]
struct ComboFile {
    r#type: &'static str,
    metadata: ComboMeta,
    combo_result: ComboResult,
    cumulative: CumulativeState,
}

#[derive(serde::Serialize)]
struct PhaseFile {
    r#type: &'static str,
    metadata: PhaseMeta,
    cumulative: CumulativeState,
}

#[derive(serde::Serialize)]
struct RunMeta {
    phase: usize, combo_index: usize, run_index: usize,
    flags: Vec<String>, runs_per_combo: usize,
}

#[derive(serde::Serialize)]
struct ComboMeta {
    phase: usize, combo_index: usize,
    flags: Vec<String>, runs_completed: usize,
}

#[derive(serde::Serialize)]
struct PhaseMeta {
    phase: usize, combos_completed: usize,
}

#[derive(serde::Serialize)]
struct RunResult { observed_opcodes: Vec<u8> }

#[derive(serde::Serialize)]
struct ComboResult { observed_opcodes: Vec<u8> }

#[derive(serde::Serialize, Clone)]
struct CumulativeState {
    discovered: Vec<DiscoveredEntry>,
    warnings: Vec<WarningEntry>,
    total_event_types: usize,
    discovered_count: usize,
}

#[derive(serde::Serialize, Clone)]
struct DiscoveredEntry {
    opcode: u8,
    event_name: String,
    class_name: String,
    best_size: usize,
    combinations: Vec<Vec<String>>,
    observed_versions: Vec<u8>,
}

#[derive(serde::Serialize)]
struct FinalJson {
    flags: Vec<FlagInfo>,
    discovered: Vec<DiscoveredEntry>,
    undiscovered: Vec<UndiscoveredEntry>,
    warnings: Vec<WarningEntry>,
}

#[derive(serde::Serialize)]
struct FlagInfo { index: usize, name: String, enable_flags: Option<String>, group_mask: Option<String> }

#[derive(serde::Serialize)]
struct UndiscoveredEntry { opcode: u8, event_name: String, class_name: String }

// ── File writers ──────────────────────────────────────────────────

pub(crate) fn write_run_file(
    dir: &Path, phase: usize, ci: usize, ri: usize,
    flags: &[Flag], indices: &[usize],
    run_opcodes: &HashSet<u8>,
    discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
    observed: &ObservedTrace,
) {
    let path = dir.join(format!("phase_{}_combo_{}_run_{}.json", phase, ci, ri));
    let file = RunFile {
        r#type: "run",
        metadata: RunMeta {
            phase, combo_index: ci, run_index: ri,
            flags: indices.iter().map(|i| flags[*i].name.clone()).collect(),
            runs_per_combo: crate::discovery::RUNS_PER_COMBO,
        },
        run_result: RunResult {
            observed_opcodes: sorted_vec(run_opcodes),
        },
        cumulative: build_cumulative(flags, discovered, event_types, observed),
    };
    write_json(&path, &file);
}

pub(crate) fn write_combo_file(
    dir: &Path, phase: usize, ci: usize,
    flags: &[Flag], indices: &[usize], runs_completed: usize,
    combo_opcodes: &HashSet<u8>,
    discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
    observed: &ObservedTrace,
) {
    let path = dir.join(format!("phase_{}_combo_{}.json", phase, ci));
    let file = ComboFile {
        r#type: "combo",
        metadata: ComboMeta {
            phase, combo_index: ci,
            flags: indices.iter().map(|i| flags[*i].name.clone()).collect(),
            runs_completed,
        },
        combo_result: ComboResult {
            observed_opcodes: sorted_vec(combo_opcodes),
        },
        cumulative: build_cumulative(flags, discovered, event_types, observed),
    };
    write_json(&path, &file);
}

pub(crate) fn write_phase_file(
    dir: &Path, phase: usize, combos_completed: usize,
    flags: &[Flag],
    discovered: &HashMap<u8, EventDiscovery>, event_types: &[EventTypeInfo],
    observed: &ObservedTrace,
) {
    let path = dir.join(format!("phase_{}.json", phase));
    let file = PhaseFile {
        r#type: "phase",
        metadata: PhaseMeta { phase, combos_completed },
        cumulative: build_cumulative(flags, discovered, event_types, observed),
    };
    write_json(&path, &file);
}

// ── Final save ────────────────────────────────────────────────────

pub(crate) fn save_final(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag], event_types: &[EventTypeInfo], dir: &Path,
    observed: &ObservedTrace,
) {
    let flag_infos: Vec<FlagInfo> = flags.iter().enumerate().map(|(i, f)| FlagInfo {
        index: i, name: f.name.clone(),
        enable_flags: f.enable_flags.map(|v| format!("0x{:08X}", v)),
        group_mask: f.group_mask.map(|v| format!("0x{:08X}", v)),
    }).collect();

    let mut disc: Vec<DiscoveredEntry> = discovered.iter().map(|(op, d)| {
        let et = event_types.iter().find(|et| et.opcode == *op);
        DiscoveredEntry {
            opcode: *op,
            event_name: et.map(|e| e.event_name.to_string()).unwrap_or_default(),
            class_name: et.map(|e| e.class_name.to_string()).unwrap_or_default(),
            best_size: d.best_size,
            combinations: d.combinations.iter()
                .map(|c| c.iter().map(|i| flags[*i].name.clone()).collect()).collect(),
            observed_versions: d.observed_versions.iter().copied().collect(),
        }
    }).collect();
    disc.sort_by_key(|e| e.opcode);

    let found: HashSet<u8> = discovered.keys().copied().collect();
    let mut un: Vec<UndiscoveredEntry> = event_types.iter()
        .filter(|et| !found.contains(&et.opcode))
        .map(|et| UndiscoveredEntry { opcode: et.opcode, event_name: et.event_name.into(), class_name: et.class_name.into() })
        .collect();
    un.sort_by_key(|e| e.opcode);

    let warnings = build_warnings(observed, flags);

    let json = FinalJson { flags: flag_infos, discovered: disc, undiscovered: un, warnings: warnings.clone() };
    write_json(&dir.join(RESULTS_FILE), &json);

    // Text summary
    let txt_path = dir.join("flag_discovery_results.txt");
    let mut txt = String::from("=== FLAG DISCOVERY RESULTS ===\n\nFlags tested:\n");
    for (i, f) in flags.iter().enumerate() {
        txt.push_str(&format!("  [{}] {} (EF={:?}, GM={:?})\n", i, f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)), f.group_mask.map(|v| format!("0x{:08X}", v))));
    }
    txt.push('\n');

    let mut by_size: HashMap<usize, Vec<&DiscoveredEntry>> = HashMap::new();
    for e in &json.discovered { by_size.entry(e.best_size).or_default().push(e); }
    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();
    for size in &sizes {
        txt.push_str(&format!("--- Size {} ---\n", size));
        for e in &by_size[size] {
            for c in &e.combinations {
                txt.push_str(&format!("  {} -> {} [{}] (Opcode={}, V={{{}}})\n",
                    c.join(" + "), e.event_name, e.class_name, e.opcode,
                    e.observed_versions.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")));
            }
        }
        txt.push('\n');
    }
    if !json.undiscovered.is_empty() {
        txt.push_str("--- Undiscovered Events ---\n");
        for e in &json.undiscovered {
            txt.push_str(&format!("  {} [{}] (Opcode={})\n", e.event_name, e.class_name, e.opcode));
        }
    }

    // Warnings section
    if !warnings.is_empty() {
        txt.push_str("\n--- Warnings ---\n");
        let unknown: Vec<&WarningEntry> = warnings.iter()
            .filter(|w| matches!(w.kind, WarningKind::UnknownOpcode))
            .collect();
        let higher: Vec<&WarningEntry> = warnings.iter()
            .filter(|w| matches!(w.kind, WarningKind::HigherVersion { .. }))
            .collect();

        if !unknown.is_empty() {
            txt.push_str("Unknown opcode events:\n");
            for w in &unknown {
                txt.push_str(&format!(
                    "  opcode={} version={} (count={}, class=\"{}\", name=\"{}\", first combo: [{}])\n",
                    w.opcode, w.version, w.count, w.class_name, w.event_name,
                    w.first_combo.join(" + ")
                ));
            }
        }
        if !higher.is_empty() {
            txt.push_str("Version higher than known:\n");
            for w in &higher {
                let max = match &w.kind { WarningKind::HigherVersion { max_known } => max_known, _ => unreachable!() };
                txt.push_str(&format!(
                    "  {} [{}] opcode={} version={} (max known V{}, count={}, first combo: [{}])\n",
                    w.event_name, w.class_name, w.opcode, w.version, max, w.count,
                    w.first_combo.join(" + ")
                ));
            }
        }
        txt.push('\n');
    }

    write_str(&txt_path, &txt);
}

// ── Display ───────────────────────────────────────────────────────

pub(crate) fn display(
    discovered: &HashMap<u8, EventDiscovery>,
    flags: &[Flag], event_types: &[EventTypeInfo],
    observed: &ObservedTrace,
) {
    log::info!("\n=== FLAG DISCOVERY RESULTS ===");

    let mut by_size: HashMap<usize, Vec<(u8, &EventDiscovery)>> = HashMap::new();
    for (op, disc) in discovered { by_size.entry(disc.best_size).or_default().push((*op, disc)); }
    let mut sizes: Vec<usize> = by_size.keys().copied().collect();
    sizes.sort();

    let mut covered: HashSet<u8> = HashSet::new();
    for size in &sizes {
        log::info!("\n--- Size {} ---", size);
        let mut combos_map: HashMap<Vec<usize>, Vec<u8>> = HashMap::new();
        for (op, disc) in &by_size[size] {
            for c in &disc.combinations { combos_map.entry(c.clone()).or_default().push(*op); }
        }
        let mut sorted: Vec<_> = combos_map.into_iter().collect();
        sorted.sort_by(|a, b| combo_key(flags, &a.0).cmp(&combo_key(flags, &b.0)));
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
                    let vers = vers_str(discovered.get(op));
                    if let Some(et) = et {
                        log::info!("    {} [{}] (Opcode={}, Versions={{{}}})", et.event_name, et.class_name, op, vers);
                    }
                }
            }
        }
        for (op, _) in &by_size[size] { covered.insert(*op); }
    }

    let all: HashSet<u8> = event_types.iter().map(|et| et.opcode).collect();
    let miss: Vec<u8> = all.difference(&covered).copied().collect::<Vec<_>>().into_iter().collect();
    if !miss.is_empty() {
        let mut s = miss; s.sort();
        log::warn!("\n--- Undiscovered Events ---\nThe following {} event type(s) were never observed:", s.len());
        for op in s {
            if let Some(et) = event_types.iter().find(|et| et.opcode == op) {
                log::warn!("  {} [{}] (Opcode={})", et.event_name, et.class_name, op);
            }
        }
    }

    // Display warnings
    let warnings = build_warnings(observed, flags);
    if !warnings.is_empty() {
        log::warn!("\n--- Warnings ---");
        for w in &warnings {
            match &w.kind {
                WarningKind::UnknownOpcode => {
                    log::warn!(
                        "Unknown opcode: opcode={} version={} count={} class=\"{}\" name=\"{}\" first combo: [{}]",
                        w.opcode, w.version, w.count, w.class_name, w.event_name,
                        w.first_combo.join(" + ")
                    );
                }
                WarningKind::HigherVersion { max_known } => {
                    log::warn!(
                        "Version higher than known: {} [{}] opcode={} version={} max known V{} count={} first combo: [{}]",
                        w.event_name, w.class_name, w.opcode, w.version, max_known, w.count,
                        w.first_combo.join(" + ")
                    );
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────

fn build_cumulative(
    flags: &[Flag], discovered: &HashMap<u8, EventDiscovery>,
    event_types: &[EventTypeInfo], observed: &ObservedTrace,
) -> CumulativeState {
    let total = event_types.len();
    let mut entries: Vec<DiscoveredEntry> = discovered.iter().map(|(op, d)| {
        let et = event_types.iter().find(|et| et.opcode == *op);
        DiscoveredEntry {
            opcode: *op,
            event_name: et.map(|e| e.event_name.to_string()).unwrap_or_default(),
            class_name: et.map(|e| e.class_name.to_string()).unwrap_or_default(),
            best_size: d.best_size,
            combinations: d.combinations.iter()
                .map(|c| c.iter().map(|i| flags[*i].name.clone()).collect()).collect(),
            observed_versions: d.observed_versions.iter().copied().collect(),
        }
    }).collect();
    entries.sort_by_key(|e| e.opcode);
    let warnings = build_warnings(observed, flags);
    CumulativeState { discovered: entries, warnings, total_event_types: total, discovered_count: discovered.len() }
}

fn sorted_vec(set: &HashSet<u8>) -> Vec<u8> { let mut v: Vec<u8> = set.iter().copied().collect(); v.sort(); v }
fn combo_key(flags: &[Flag], indices: &[usize]) -> String { indices.iter().map(|i| flags[*i].name.as_str()).collect::<Vec<_>>().join("+") }
fn vers_str(d: Option<&EventDiscovery>) -> String {
    d.map(|d| { let mut v: Vec<u8> = d.observed_versions.iter().copied().collect(); v.sort();
        v.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",") }).unwrap_or_default()
}

fn write_json<T: serde::Serialize>(path: &Path, val: &T) {
    if let Ok(s) = serde_json::to_string_pretty(val) {
        if let Err(e) = fs::write(path, &s) { log::warn!("Failed to write {}: {}", path.display(), e); }
    }
}

fn write_str(path: &Path, content: &str) {
    if let Err(e) = fs::write(path, content) { log::error!("Failed to write {}: {}", path.display(), e); }
    else { log::info!("Saved to {}", path.display()); }
}
