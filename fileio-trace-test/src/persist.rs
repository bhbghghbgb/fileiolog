use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::events::{EVENT_REGISTRY, FileIoRawEvent};

/// Serialized event key: "opcode:version"
fn event_key(opcode: u8, version: u8) -> String {
    format!("{}:{}", opcode, version)
}

/// Parse an event key back to (opcode, version)
fn parse_event_key(key: &str) -> Option<(u8, u8)> {
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let opcode = parts[0].parse::<u8>().ok()?;
    let version = parts[1].parse::<u8>().ok()?;
    Some((opcode, version))
}

/// Top-level persisted data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedData {
    /// Total number of runs that contributed to these counts
    pub total_runs: usize,
    /// config_name -> { "opcode:version" -> cumulative_count }
    pub config_events: HashMap<String, HashMap<String, usize>>,
}

impl PersistedData {
    fn new() -> Self {
        Self {
            total_runs: 0,
            config_events: HashMap::new(),
        }
    }
}

/// Load persisted data from disk. Returns empty data if file doesn't exist.
pub fn load(path: &Path) -> PersistedData {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<PersistedData>(&content) {
            Ok(data) => {
                log::info!(
                    "Loaded persisted results from {} ({} prior runs)",
                    path.display(),
                    data.total_runs
                );
                data
            }
            Err(e) => {
                log::warn!(
                    "Failed to parse {}: {}. Starting fresh.",
                    path.display(),
                    e
                );
                PersistedData::new()
            }
        },
        Err(_) => {
            log::info!("No persisted results found at {}. Starting fresh.", path.display());
            PersistedData::new()
        }
    }
}

/// Save persisted data to disk
pub fn save(path: &Path, data: &PersistedData) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => {
            if let Err(e) = fs::write(path, json) {
                log::error!("Failed to write {}: {}", path.display(), e);
            } else {
                log::info!("Persisted results to {}", path.display());
            }
        }
        Err(e) => {
            log::error!("Failed to serialize persisted data: {}", e);
        }
    }
}

/// Compute per-config event counts from raw events
pub fn compute_counts(events: &[FileIoRawEvent]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for event in events {
        let key = event_key(event.opcode, event.version);
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

/// Merge current run's results into the persisted data.
///
/// For every (config, event) pair:
///   - If it exists in persisted, add the current count to it
///   - If it only exists in current run, insert it
///   - If it only exists in persisted (not seen this run), keep it with count 0 for this run
pub fn merge(persisted: &mut PersistedData, current: &HashMap<String, HashMap<String, usize>>) {
    persisted.total_runs += 1;
    for (config_name, event_counts) in current {
        let entry = persisted
            .config_events
            .entry(config_name.clone())
            .or_insert_with(HashMap::new);
        for (event_key, count) in event_counts {
            *entry.entry(event_key.clone()).or_insert(0) += count;
        }
    }
}

/// Display the cumulative results
pub fn display(persisted: &PersistedData, current: &HashMap<String, HashMap<String, usize>>) {
    log::info!("");
    log::info!("=== CUMULATIVE RESULTS ({} total runs) ===", persisted.total_runs);

    // Collect all known event keys across all configs
    let mut all_event_configs: HashMap<String, Vec<(String, usize, usize)>> = HashMap::new();

    // event_key -> vec of (config_name, current_count, cumulative_count)
    for (config_name, persisted_events) in &persisted.config_events {
        for (ek, &cumulative_count) in persisted_events {
            let current_count = current
                .get(config_name)
                .and_then(|ce| ce.get(ek))
                .copied()
                .unwrap_or(0);

            all_event_configs
                .entry(ek.clone())
                .or_default()
                .push((config_name.clone(), current_count, cumulative_count));
        }
    }

    // Sort events by opcode and version
    let mut sorted: Vec<_> = all_event_configs.into_iter().collect();
    sorted.sort_by_key(|(ek, _)| {
        let (op, ver) = parse_event_key(ek).unwrap_or((0, 0));
        (op, ver)
    });

    for (ek, mut config_entries) in sorted {
        let (opcode, version) = match parse_event_key(&ek) {
            Some(v) => v,
            None => continue,
        };

        let label = if let Some(known) = EVENT_REGISTRY.get(&(opcode, version)) {
            format!(
                "{} [{}] (Opcode={}, Version={})",
                known.event_name, known.class_name, opcode, version
            )
        } else {
            format!("UNKNOWN (Opcode={}, Version={})", opcode, version)
        };

        log::info!("");
        log::info!("{}", label);
        config_entries.sort_by(|a, b| a.0.cmp(&b.0));

        for (config_name, current_count, cumulative_count) in &config_entries {
            if *current_count > 0 {
                log::info!(
                    "    {}: {} this run, {} cumulative",
                    config_name,
                    current_count,
                    cumulative_count
                );
            } else {
                log::info!(
                    "    {}: 0 this run, {} cumulative (persisted)",
                    config_name,
                    cumulative_count
                );
            }
        }
    }

    // Warn about defined events that were never received in any run
    let received_keys: std::collections::HashSet<String> =
        persisted.config_events.values().flat_map(|m| m.keys().cloned()).collect();

    let mut unreceived: Vec<_> = EVENT_REGISTRY
        .iter()
        .filter(|((op, ver), _)| !received_keys.contains(&format!("{}:{}", op, ver)))
        .collect();
    unreceived.sort_by_key(|((op, ver), _)| (*op, *ver));

    if !unreceived.is_empty() {
        log::warn!("");
        log::warn!("Defined events NEVER received across all runs:");
        log::warn!("-----------------------------------------------");
        for ((opcode, version), def) in &unreceived {
            log::warn!(
                "  {} [{}] (Opcode={}, Version={})",
                def.event_name,
                def.class_name,
                opcode,
                version
            );
        }
    }
}
