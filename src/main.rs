mod etw;
mod event;
mod providers;

use std::{process::exit, time::Duration};

use ferrisetw::trace::{UserTrace, stop_trace_by_name};

fn main() {
    // 1. Initialize env_logger
    // Set the default log level to 'info' if the RUST_LOG environment variable isn't specified.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let session_name = "FileIoLog";

    log::info!("Starting up ETW Monitor Application...");

    // 3. Pre-emptively stop any orphan sessions from previous crashes
    log::info!(
        "Checking for any lingering ETW sessions named '{}'...",
        session_name
    );
    match stop_trace_by_name(session_name) {
        Ok(_) => {
            log::warn!(
                "Found and successfully terminated a stale trace session: '{}'.",
                session_name
            );
        }
        Err(e) => {
            // Note: If the session didn't exist, control_trace_by_name typically returns
            // an error representing ERROR_WMI_INSTANCE_NOT_FOUND (0x1068). We treat this as safe.
            log::info!(
                "No lingering session detected (or it was clean). Error code ignored: {:?}",
                e
            );
        }
    }

    // 4. Construct the provider callback logic
    let file_provider = providers::kernel_file::build_provider();

    // 5. Build and initialize the UserTrace
    log::info!("Creating new ETW session: '{}'...", session_name);
    let trace_result = UserTrace::new()
        .named(String::from(session_name))
        .enable(file_provider)
        .start_and_process();

    // 6. Securely handle any activation errors without unwrap() panicking
    let trace = match trace_result {
        Ok(t) => t,
        Err(err) => {
            log::error!("Failed to start the ETW trace session: {:?}", err);
            log::error!("Application exiting securely.");
            exit(1);
        }
    };

    log::info!("ETW Trace session active and running. Monitoring logs for 10 seconds...");
    std::thread::sleep(Duration::from_secs(10));

    // 7. Explicitly stop the trace session cleanly
    log::info!("Shutting down trace session...");
    if let Err(e) = trace.stop() {
        log::error!("Failed to stop the trace session cleanly: {:?}", e);
    } else {
        log::info!("Trace session stopped safely.");
    }
}
