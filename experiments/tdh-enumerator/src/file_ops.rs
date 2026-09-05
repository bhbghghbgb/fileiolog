use std::process::Command;
use std::time::Duration;

/// Trigger various file system operations by invoking the file-ops-trigger binary.
/// Waits for it to exit, then waits for ETW events to flush.
pub fn trigger_all_file_operations() {
    log::debug!("Invoking file-ops-trigger...");
    let bin = file_ops_trigger::bin_path();
    let output = Command::new(&bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", bin.display()));

    log::debug!("file-ops-trigger exited with status: {}", output.status);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("file-ops-trigger failed: {stderr}");
    }

    log::debug!("Waiting for ETW events to flush...");
    std::thread::sleep(Duration::from_secs(3));
}
