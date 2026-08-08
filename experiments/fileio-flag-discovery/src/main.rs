mod discovery;
mod event_types;
mod events;
mod file_ops;
mod fileio_events;
mod flags;
mod output;
mod trace_session;

use std::fs;
use std::path::Path;

const OUTPUT_DIR: &str = "flag_discovery_output";
const RESULTS_FILE: &str = "flag_discovery_results.json";

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis().init();

    log::info!("=== FileIo Flag Discovery ===");
    log::info!("Runs per combination: {}", discovery::RUNS_PER_COMBO);
    log::info!("Max combination size: {}", discovery::MAX_COMBO_SIZE);
    log::info!("Event matching is version-agnostic (any version of an opcode counts).\n");

    let flag_list = flags::build_all_flags();
    let event_list = event_types::build_event_types();

    log::info!("Flags to test ({} total):", flag_list.len());
    for (i, f) in flag_list.iter().enumerate() {
        log::info!("  [{}] {}: EF={:?} GM={:?}", i, f.name,
            f.enable_flags.map(|v| format!("0x{:08X}", v)),
            f.group_mask.map(|v| format!("0x{:08X}", v)));
    }
    log::info!("\nUnique event types (opcodes) to discover: {}", event_list.len());
    for et in &event_list {
        log::info!("  Opcode={} {} [{}]", et.opcode, et.event_name, et.class_name);
    }
    log::info!("");

    let output_dir = Path::new(OUTPUT_DIR);
    if let Err(e) = fs::create_dir_all(output_dir) {
        log::warn!("Failed to create output directory: {}", e);
    }

    let discovery = discovery::discover(&flag_list, &event_list, output_dir);
    output::display(&discovery, &flag_list, &event_list);
    output::save_final(&discovery, &flag_list, &event_list, output_dir);
}
