fn main() {
    // If we are building the GUI, let Tauri's builder handle the Windows manifest
    // to avoid duplicate resource errors with winresource.
    if std::env::var("CARGO_FEATURE_GUI").is_ok() {
        #[cfg(target_os = "windows")]
        {
            let mut windows = tauri_build::WindowsAttributes::new();
            windows = windows.app_manifest(include_str!("app.manifest"));
            tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
                .unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            tauri_build::build();
        }
    } else {
        // If we are building the CLI (no GUI feature), manually compile the manifest
        #[cfg(target_os = "windows")]
        {
            let mut res = winresource::WindowsResource::new();
            res.set_manifest_file("app.manifest");
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resource for CLI: {}", e);
            }
        }
    }
}
