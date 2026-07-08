mod etw;
mod manager;
mod provider_event;
mod providers;

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

    // 3. Initialize the manager
    let mut etw_manager = EtwTraceManager::new("FileIoLog");

    // 4. Start the session
    if let Err(e) = etw_manager.start(shared_event_callback) {
        etw_manager
            .stop()
            .expect("Error when stopping manager due to initialization failure.");
        panic!("Application exiting due to initialization failure. {:?}", e);
    }

    log::info!("Monitoring logs for 10 seconds...");
    std::thread::sleep(Duration::from_secs(10));

    log::info!("Application work period finished. Execution exiting scope...");
    // etw_manager goes out of scope here; its Drop trait cleans up everything automatically.
}
