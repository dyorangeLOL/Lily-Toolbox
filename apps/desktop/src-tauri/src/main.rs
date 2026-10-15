#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use toolbox_core::{AppError, HealthStatus};

static STARTED: OnceLock<Instant> = OnceLock::new();

// Experiments are compiled only by an explicit feature, never the normal Shell build.
#[cfg(feature = "spike-web")]
#[path = "../../../../tools/spikes/wf02.rs"]
mod spike;

#[derive(Default)]
struct Readiness(Mutex<bool>);

#[tauri::command]
fn health_check() -> HealthStatus {
    toolbox_core::health()
}

#[tauri::command]
fn frontend_ready(state: tauri::State<'_, Readiness>) -> Result<(), AppError> {
    let mut ready = state.0.lock().map_err(|_| AppError::diagnostic_failed())?;
    if *ready {
        return Ok(());
    }
    // Test-only destination comes from the launching process, never from webpage input.
    if let Some(path) = std::env::var_os("TOOLBOX_SMOKE_REPORT") {
        let report = serde_json::json!({
            "health": toolbox_core::health(),
            "frontend_ready_ms": STARTED.get().map(|time| time.elapsed().as_millis()),
            "pid": std::process::id()
        });
        std::fs::write(path, report.to_string()).map_err(|_| AppError::diagnostic_failed())?;
    }
    *ready = true;
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    STARTED.get_or_init(Instant::now);
    #[cfg(feature = "spike-web")]
    let test_data = spike::prepare_data_directory()?;
    tauri::Builder::default()
        .manage(Readiness::default())
        .invoke_handler(tauri::generate_handler![health_check, frontend_ready])
        .setup(move |app| {
            let builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("Toolbox — WF-01")
                    .inner_size(800.0, 600.0)
                    .on_navigation(|url| {
                        let bundled =
                            url.scheme() == "tauri" || url.host_str() == Some("tauri.localhost");
                        let development = cfg!(debug_assertions)
                            && url.scheme() == "http"
                            && url.host_str() == Some("127.0.0.1")
                            && url.port() == Some(1420);
                        bundled || development
                    });
            #[cfg(feature = "spike-web")]
            let builder = if let Some(path) = &test_data {
                builder.data_directory(path.clone())
            } else {
                builder
            };
            let window = builder.build();
            #[cfg(feature = "spike-web")]
            if window.is_err() {
                spike::report_webview_failure();
                // An expected fixture/prerequisite failure must not reach Tauri's
                // setup panic handler (which can block on Windows crash dialogs).
                // Only enabled for the explicit experiment; no database exists.
                std::process::exit(2);
            }
            let window = window?;
            window.show()?;
            #[cfg(feature = "spike-web")]
            spike::maybe_start(app.handle().clone(), window.as_ref().window())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

fn main() {
    if run().is_err() {
        eprintln!("Toolbox startup failed; check platform prerequisites or test report");
        std::process::exit(2);
    }
}
