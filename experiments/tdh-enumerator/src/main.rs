mod config;
mod output;
mod session;
mod tdh;
mod types;

use clap::Parser as ClapParser;
use config::AppConfig;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::parse();

    log::info!("=== TDH Event Enumerator ===");
    log::info!("Mode: {:?}", config.mode);
    log::info!("Duration: {} seconds", config.duration);
    log::info!("Output prefix: {}", config.output_prefix);

    if let Err(e) = session::run_session(&config) {
        log::error!("Session failed: {:?}", e);
        std::process::exit(1);
    }

    log::info!("Done.");
}
