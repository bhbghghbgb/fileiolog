fn main() {
    // Embed the manifest requiring administrator privileges for kernel tracing
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.compile().expect("Failed to compile Windows resource");
    }
}