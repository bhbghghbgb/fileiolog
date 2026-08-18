use std::process::Command;

/// Trigger various file system operations by invoking the file-ops-trigger binary
pub fn trigger_all_file_operations() {
    let bin = file_ops_trigger::bin_path();
    let output = Command::new(&bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {}: {e}", bin.display()));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("file-ops-trigger failed: {stderr}");
    }
}
