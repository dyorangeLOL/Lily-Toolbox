fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "health_check",
            "frontend_ready",
            "core_dispatch",
        ]),
    ))
    .expect("failed to build Tauri application metadata");
}
