mod etw;
mod manager;
mod perfinfo_groupmask;
mod provider_event;
mod providers;
mod rundown;

use std::time::Duration;

use crate::{manager::EtwTraceManager, perfinfo_groupmask::*, provider_event::ProviderEvent};

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

    // 2. Define the unified callback (For now logging; later pushing to ringbuf SPSC)
    let shared_event_callback = |event: ProviderEvent| {
        log::info!("Received Event: {:?}", event);
    };

    // 3. Build and start the session
    //
    // Example A: User trace (default — no extended flags needed)
    // let _session = EtwTraceManager::new("FileIoLog")
    //     .start(shared_event_callback)
    //     .expect("Failed to start ETW trace session");

    // Example B: Kernel trace with PERFINFO_GROUPMASK for minifilter events.
    // The group_mask! macro builds a [u32; 8] array from flag constants.
    // Only the group containing the flags is populated — no boilerplate for all 8 groups.
    let mask = group_mask![PERF_FLT_IO_INIT, PERF_FLT_IO, PERF_FLT_IO_FAILURE];

    let _session = EtwTraceManager::new("FileIoLog")
        .with_group_mask(mask)
        .with_enable_flags(PERF_FILE_IO_INIT | PERF_FILE_IO | PERF_DISK_FILE_IO)
        .start(shared_event_callback)
        .expect("Failed to start ETW trace session");

    log::info!("Monitoring logs for 3 seconds...");
    std::thread::sleep(Duration::from_secs(3));

    log::info!("Application work period finished. Execution exiting scope...");
    // _session goes out of scope here; its Drop trait cleans up everything automatically.
}
