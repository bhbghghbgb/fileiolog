mod discovery;
mod event_types;
mod events;
mod file_ops;
mod fileio_events;
mod flags;
mod output;
mod trace_session;

use std::fs;
use std::path::PathBuf;

use clap::Parser as ClapParser;

#[derive(Debug, ClapParser)]
#[command(name = "fileio-flag-discovery")]
#[command(about = "Discover which EnableFlags/GROUPMASK enable which FileIo events")]
struct Args {
    /// Output directory for results
    #[arg(short, long, default_value = "output")]
    output: PathBuf,
}

const RESULTS_FILE: &str = "flag_discovery_results.json";

fn main() {
    let args = Args::parse();
    let output_dir = &args.output;

    let _ = fs::create_dir_all(output_dir);
    fileiolog::logging::init_logging(output_dir, "fileio-flag-discovery");

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

    if let Err(e) = fs::create_dir_all(output_dir) {
        log::warn!("Failed to create output directory: {}", e);
    }

    let result = discovery::discover(&flag_list, &event_list, output_dir);
    output::display(&result.per_opcode, &flag_list, &event_list, &result.observed);
    output::save_final(&result.per_opcode, &flag_list, &event_list, output_dir, &result.observed);
}
