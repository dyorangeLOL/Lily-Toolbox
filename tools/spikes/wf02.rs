//! WF-02 real Windows/WebView2 experiment, not production browser business logic.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::webview::{DownloadEvent, PageLoadEvent, WebviewBuilder};
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl, Window};

fn test_root() -> Option<PathBuf> {
    std::env::var_os("TOOLBOX_PORTABLE_ROOT")
        .or_else(|| std::env::var_os("TOOLBOX_SPIKE_DATA_ROOT"))
        .map(PathBuf::from)
}

fn report(value: serde_json::Value) {
    if let Some(path) = std::env::var_os("TOOLBOX_SPIKE_REPORT") {
        // Destination is supplied by the test launcher, not remote content.
        if std::fs::write(path, value.to_string()).is_err() {
            eprintln!("Unable to write spike evidence");
        }
    }
}

pub fn prepare_data_directory() -> Result<Option<PathBuf>, std::io::Error> {
    let Some(root) = test_root() else {
        return Ok(None);
    };
    let data = root.join("data");
    let result = (|| {
        std::fs::create_dir_all(&data)?;
        let probe = data.join(format!("write-probe-{}", std::process::id()));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)?;
        drop(file);
        // Only remove the file we successfully created ourselves.
        std::fs::remove_file(&probe)?;
        let shell = data.join("webview").join("shell");
        std::fs::create_dir_all(&shell)?;
        Ok::<PathBuf, std::io::Error>(shell)
    })();
    match result {
        Ok(path) => Ok(Some(path)),
        Err(error) => {
            report(serde_json::json!({"status":"FAIL","code":"DATA_DIRECTORY_NOT_WRITABLE"}));
            Err(error)
        }
    }
}

fn wait_for(flag: &AtomicBool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !flag.load(Ordering::SeqCst) {
        if Instant::now() > deadline {
            return Err("WEBVIEW_EVENT_TIMEOUT".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}

struct Tab {
    view: Webview,
    loaded: Arc<AtomicBool>,
}

fn create_tab(
    window: &Window,
    base: &tauri::Url,
    root: &std::path::Path,
    slot: usize,
    step: usize,
    page_loads: Arc<AtomicUsize>,
    downloaded: Arc<AtomicBool>,
) -> Result<Tab, String> {
    let group = if slot % 2 == 0 { "A" } else { "B" };
    let directory = root.join("data").join("webview").join(group);
    std::fs::create_dir_all(&directory).map_err(|_| "PROFILE_DIRECTORY_FAILED")?;
    let label = format!("spike-tab-{slot}");
    let mut url = base.clone();
    url.query_pairs_mut()
        .append_pair("slot", &slot.to_string())
        .append_pair("profile", group)
        .append_pair("step", &step.to_string());
    let loaded = Arc::new(AtomicBool::new(false));
    let page_flag = loaded.clone();
    let origin = base.origin();
    let download_origin = origin.clone();
    let download_destination = root.join("data").join("fixture-download.txt");
    let expected_destination = download_destination.clone();
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(url))
        .data_directory(directory)
        .on_navigation(move |next| next.origin() == origin)
        .on_page_load(move |_, payload| {
            if payload.event() == PageLoadEvent::Finished {
                page_loads.fetch_add(1, Ordering::SeqCst);
                page_flag.store(true, Ordering::SeqCst);
            }
        })
        .on_download(move |_, event| match event {
            DownloadEvent::Requested { url, destination } => {
                if url.origin() != download_origin || url.path() != "/download" {
                    return false;
                }
                *destination = download_destination.clone();
                true
            }
            DownloadEvent::Finished { path, success, .. } => {
                let correct = success
                    && path.as_ref() == Some(&expected_destination)
                    && std::fs::read(&expected_destination)
                        .is_ok_and(|bytes| bytes == b"TOOLBOX-DOWNLOAD-FIXTURE");
                downloaded.store(correct, Ordering::SeqCst);
                true
            }
            _ => false,
        });
    // Called from an independent worker thread, per Tauri's Windows deadlock guidance.
    let view = window
        .add_child(
            builder,
            LogicalPosition::new(0.0, 180.0),
            LogicalSize::new(760.0, 350.0),
        )
        .map_err(|_| "CREATE_CHILD_FAILED")?;
    wait_for(&loaded)?;
    std::thread::sleep(Duration::from_millis(120));
    view.hide().map_err(|_| "HIDE_CHILD_FAILED")?;
    Ok(Tab { view, loaded })
}

fn execute(app: &AppHandle, window: &Window) -> Result<serde_json::Value, String> {
    let root = test_root().ok_or("SPIKE_DATA_ROOT_REQUIRED")?;
    let base: tauri::Url = std::env::var("TOOLBOX_SPIKE_URL")
        .map_err(|_| "SPIKE_URL_REQUIRED")?
        .parse()
        .map_err(|_| "SPIKE_URL_INVALID")?;
    if base.scheme() != "http" || base.host_str() != Some("127.0.0.1") {
        return Err("ONLY_CONTROLLED_LOOPBACK_ALLOWED".into());
    }
    let count: usize = std::env::var("TOOLBOX_SPIKE_TABS")
        .unwrap_or_else(|_| "10".into())
        .parse()
        .map_err(|_| "TAB_COUNT_INVALID")?;
    let operations: usize = std::env::var("TOOLBOX_SPIKE_OPERATIONS")
        .unwrap_or_else(|_| "100".into())
        .parse()
        .map_err(|_| "OPERATION_COUNT_INVALID")?;
    if ![0, 1, 5, 10].contains(&count)
        || ![0, 100].contains(&operations)
        || (operations > 0 && count != 10)
    {
        return Err("SPIKE_LIMIT_INVALID".into());
    }
    let page_loads = Arc::new(AtomicUsize::new(0));
    let downloaded = Arc::new(AtomicBool::new(false));
    let mut tabs = Vec::new();
    for slot in 0..count {
        tabs.push(create_tab(
            window,
            &base,
            &root,
            slot,
            0,
            page_loads.clone(),
            downloaded.clone(),
        )?);
    }
    if window.webviews().len() != count + 1 || app.windows().len() != 1 {
        return Err("NOT_ONE_WINDOW_WITH_REAL_CHILDREN".into());
    }
    let mut switches = 0;
    let mut resizes = 0;
    let mut navigations = 0;
    let mut recreates = 0;
    for step in 1..=operations {
        let slot = (step - 1) % count;
        match (step - 1) % 4 {
            0 => {
                for tab in &tabs {
                    tab.view.hide().map_err(|_| "SWITCH_HIDE_FAILED")?;
                }
                tabs[slot].view.show().map_err(|_| "SWITCH_SHOW_FAILED")?;
                tabs[slot].view.set_focus().map_err(|_| "FOCUS_FAILED")?;
                switches += 1;
            }
            1 => {
                let width = if (step / 4) % 2 == 0 { 880.0 } else { 800.0 };
                window
                    .set_size(LogicalSize::new(width, 640.0))
                    .map_err(|_| "WINDOW_RESIZE_FAILED")?;
                tabs[slot]
                    .view
                    .set_size(LogicalSize::new(width - 40.0, 420.0))
                    .map_err(|_| "CHILD_RESIZE_FAILED")?;
                resizes += 1;
            }
            2 => {
                let mut url = tabs[slot].view.url().map_err(|_| "CURRENT_URL_FAILED")?;
                url.query_pairs_mut()
                    .append_pair("cycle", &step.to_string());
                tabs[slot].loaded.store(false, Ordering::SeqCst);
                tabs[slot]
                    .view
                    .navigate(url)
                    .map_err(|_| "NAVIGATE_FAILED")?;
                wait_for(&tabs[slot].loaded)?;
                std::thread::sleep(Duration::from_millis(120));
                navigations += 1;
            }
            _ => {
                tabs[slot].view.close().map_err(|_| "CLOSE_CHILD_FAILED")?;
                tabs[slot] = create_tab(
                    window,
                    &base,
                    &root,
                    slot,
                    step,
                    page_loads.clone(),
                    downloaded.clone(),
                )?;
                recreates += 1;
            }
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    if operations > 0 {
        tabs[0]
            .view
            .show()
            .map_err(|_| "DOWNLOAD_VIEW_SHOW_FAILED")?;
        tabs[0]
            .view
            .eval("document.getElementById('download').click()")
            .map_err(|_| "DOWNLOAD_TRIGGER_FAILED")?;
        wait_for(&downloaded)?;
    }
    if count > 0 {
        tabs[0].view.show().map_err(|_| "FINAL_SHOW_FAILED")?;
    }
    Ok(serde_json::json!({
        "status":"PASS", "tabs":count, "operations":operations,
        "switches":switches, "resizes":resizes, "navigations":navigations, "close_recreate":recreates,
        "page_loads":page_loads.load(Ordering::SeqCst), "download_verified":downloaded.load(Ordering::SeqCst),
        "one_window":app.windows().len()==1, "attached_webviews":window.webviews().len(),
        "profile_strategy":"separate UDF per workspace; shared UDF/named profiles NOT VERIFIED"
    }))
}

pub fn maybe_start(app: AppHandle, window: Window) -> Result<(), std::io::Error> {
    if std::env::var_os("TOOLBOX_SPIKE_URL").is_none() {
        return Ok(());
    }
    std::thread::spawn(move || match execute(&app, &window) {
        Ok(result) => report(result),
        Err(code) => report(serde_json::json!({"status":"FAIL","code":code})),
    });
    Ok(())
}
