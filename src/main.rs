mod etw;
mod manager;
mod provider_event;
mod providers;
mod rundown;

use std::time::Duration;

use crate::{manager::EtwTraceManager, provider_event::ProviderEvent};

fn main() {
    // 1. Initialize env_logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting up ETW Monitor Application...");

    // 2. Define the unified callback (For now logging; later pushing to ringbuf SPSC)
    let shared_event_callback = |event: ProviderEvent| {
        log::info!("Received Event: {:?}", event);
    };

    // 3. Build and start the session
    let _session = EtwTraceManager::new("FileIoLog")
        .start(shared_event_callback)
        .expect("Failed to start ETW trace session");

    log::info!("Monitoring logs for 10 seconds...");
    std::thread::sleep(Duration::from_secs(1));

    log::info!("Application work period finished. Execution exiting scope...");
    // _session goes out of scope here; its Drop trait cleans up everything automatically.
}
