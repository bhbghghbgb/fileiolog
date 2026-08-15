mod etw;
mod manager;
mod provider_event;
mod providers;
mod rundown;

use std::time::Duration;

use crate::{manager::EtwTraceManager, provider_event::ProviderEvent};

fn main() {
    // 1. Initialize env_logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "info"
        },
    ))
    .init();
    log::info!("Starting up ETW Monitor Application...");

    // 2. Define the unified callback
    //    The callback is wrapped in Arc internally and will be called from
    //    both the UserTrace and KernelTrace threads in parallel.
    let shared_event_callback = |event: ProviderEvent| {
        log::info!("Received Event: {:?}", event);
    };

    // 3. Build and start both sessions (UserTrace + KernelTrace)
    let _session = EtwTraceManager::new("FileIoLog")
        .start(shared_event_callback)
        .expect("Failed to start ETW trace sessions");

    log::info!("Monitoring logs for 3 seconds...");
    std::thread::sleep(Duration::from_secs(3));

    log::info!("Application work period finished. Execution exiting scope...");
    // _session goes out of scope here; its Drop trait cleans up both sessions.
}
