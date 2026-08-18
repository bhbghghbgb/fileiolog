use std::path::PathBuf;

/// Returns the path to the `file-ops-trigger` binary.
///
/// Resolves relative to the workspace root (two levels up from this crate's manifest).
pub fn bin_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .expect("no parent")
        .parent()
        .expect("no workspace root");
    let ext = if cfg!(windows) { ".exe" } else { "" };
    workspace_root
        .join("target")
        .join("debug")
        .join(format!("file-ops-trigger{ext}"))
}
