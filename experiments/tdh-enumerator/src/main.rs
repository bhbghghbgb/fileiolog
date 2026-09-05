mod config;
mod file_ops;
mod output;
mod session;
mod tdh;
mod types;

use std::fs;

use clap::Parser as ClapParser;
use config::AppConfig;

fn main() {
    let config = AppConfig::parse();

    let _ = fs::create_dir_all(&config.output);
    fileiolog::logging::init_logging(&config.output, "tdh-enumerator");

    log::info!("=== TDH Event Enumerator ===");
    log::info!("Mode: {:?}", config.mode);
    log::info!("Duration: {} seconds", config.duration);
    log::info!("Output directory: {}", config.output.display());
    log::info!("Output prefix: {}", config.output_prefix);

    if let Err(e) = session::run_session(&config) {
        log::error!("Session failed: {:?}", e);
        std::process::exit(1);
    }

    log::info!("Done.");
}
