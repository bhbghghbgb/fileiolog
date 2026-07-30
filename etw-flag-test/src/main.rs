mod events;
mod flags;
mod trace_session;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use events::EventDefs;
use flags::TestConfig;
use trace_session::EventCollector;

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("=== ETW FileIo Flag/Mask Test ===");
    log::info!("This program tests which EnableFlags and PERFINFO_GROUPMASK bits");
    log::info!("enable which FileIo event types in kernel trace sessions.");
    log::info!("");

    // Get all test cases
    let test_cases = TestConfig::fileio_test_cases();
    log::info!("Found {} test configurations to run.", test_cases.len());
    log::info!("");

    // Map: test_name -> set of event keys seen
    let mut results: HashMap<String, HashSet<(u16, u8)>> = HashMap::new();

    // Global unknown event warnings
    let mut all_unknown: HashSet<(u16, u8)> = HashSet::new();

    for (i, (name, config)) in test_cases.iter().enumerate() {
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        log::info!(
            "Test {}/{}: {}",
            i + 1,
            test_cases.len(),
            name
        );
        log::info!("  Config: {}", config.name());

        // Create a fresh collector for this test
        let collector = Arc::new(Mutex::new(EventCollector::new()));

        // Run the test
        let seen = trace_session::run_single_test(config, Arc::clone(&collector));

        // Report results
        log::info!("  Events seen ({} unique, {} total):", seen.len(), {
            let coll = collector.lock().unwrap();
            coll.total_count
        });

        let mut sorted: Vec<(u16, u8)> = seen.iter().copied().collect();
        sorted.sort();

        for (id, ver) in &sorted {
            match EventDefs::lookup(*id, *ver) {
                Some(info) => {
                    log::info!(
                        "    [{} v{}] {} ({})",
                        id,
                        ver,
                        info.name,
                        info.mof_class
                    );
                }
                None => {
                    if !all_unknown.contains(&(*id, *ver)) {
                        log::warn!(
                            "    [{} v{}] UNKNOWN EVENT (first encounter)",
                            id,
                            ver
                        );
                        all_unknown.insert((*id, *ver));
                    } else {
                        log::debug!(
                            "    [{} v{}] UNKNOWN EVENT",
                            id,
                            ver
                        );
                    }
                }
            }
        }

        results.insert(name.clone(), seen);
        log::info!("");
    }

    // ── Summary ──
    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log::info!("=== SUMMARY: Flag/Mask → Event Type Mapping ===");
    log::info!("");

    // Build reverse map: event_key -> list of flag names that enable it
    let mut event_to_flags: HashMap<(u16, u8), Vec<String>> = HashMap::new();
    for (flag_name, events) in &results {
        for key in events {
            event_to_flags
                .entry(*key)
                .or_default()
                .push(flag_name.clone());
        }
    }

    // Print per-flag summary
    for (name, config) in &test_cases {
        if let Some(events) = results.get(name) {
            let mut sorted: Vec<(u16, u8)> = events.iter().copied().collect();
            sorted.sort();

            let event_names: Vec<String> = sorted
                .iter()
                .map(|(id, ver)| {
                    match EventDefs::lookup(*id, *ver) {
                        Some(info) => format!("{}({})", info.name, ver),
                        None => format!("Unknown({},{})", id, ver),
                    }
                })
                .collect();

            log::info!("{}:", name);
            log::info!("  Config: {}", config.name());
            log::info!("  Events: [{}]", event_names.join(", "));
            log::info!("");
        }
    }

    // Print per-event summary
    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    log::info!("=== Per-Event Summary ===");
    log::info!("");

    let mut all_keys: Vec<(u16, u8)> = event_to_flags.keys().copied().collect();
    all_keys.sort();
    all_keys.dedup();

    for key in &all_keys {
        let flags_list = &event_to_flags[key];
        let mut flags_sorted = flags_list.clone();
        flags_sorted.sort();

        let event_name = match EventDefs::lookup(key.0, key.1) {
            Some(info) => format!("{} ({})", info.name, info.mof_class),
            None => format!("Unknown(id={}, ver={})", key.0, key.1),
        };

        log::info!(
            "  [{} v{}] {}",
            key.0,
            key.1,
            event_name
        );
        log::info!("    Enabled by: {}", flags_sorted.join(", "));
    }

    // Print unknown events
    if !all_unknown.is_empty() {
        log::info!("");
        log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        log::info!("=== Unknown Events Observed ===");
        log::info!("These event id/version combinations were not in the known definitions.");
        let mut unknown_sorted: Vec<(u16, u8)> = all_unknown.into_iter().collect();
        unknown_sorted.sort();
        for (id, ver) in &unknown_sorted {
            log::warn!("  [{} v{}]", id, ver);
        }
    }

    log::info!("");
    log::info!("=== Test Complete ===");
}
