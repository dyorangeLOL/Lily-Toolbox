//! Public Wry Windows extension experiment: named Profiles in the Shell environment.
//! All COM objects and child controls stay on the window thread.
use super::{test_root, wait_for};
use raw_window_handle::{HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle};
use std::cell::RefCell;
use std::num::NonZeroIsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tauri::{AppHandle, Manager, Window};
use wry::{WebView, WebViewBuilder, WebViewBuilderExtWindows};

thread_local! {
    static VIEWS: RefCell<Vec<Option<WebView>>> = const { RefCell::new(Vec::new()) };
}

struct Parent(NonZeroIsize);
impl HasWindowHandle for Parent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
        // SAFETY: obtained from the still-live Tauri window; borrowed only during
        // synchronous child creation on that window's thread. Never owns the HWND.
        Ok(unsafe {
            WindowHandle::borrow_raw(RawWindowHandle::Win32(Win32WindowHandle::new(self.0)))
        })
    }
}

fn on_ui<T: Send + 'static>(
    main: &tauri::Webview,
    action: impl FnOnce(tauri::webview::PlatformWebview) -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    let (sender, receiver) = mpsc::channel();
    main.with_webview(move |platform| {
        let _ = sender.send(action(platform));
    })
    .map_err(|_| "UI_DISPATCH_FAILED")?;
    receiver
        .recv_timeout(Duration::from_secs(45))
        .map_err(|_| "UI_REPLY_TIMEOUT")?
}

fn bounds(width: f64) -> wry::Rect {
    wry::Rect {
        position: wry::dpi::LogicalPosition::new(0.0, 180.0).into(),
        size: wry::dpi::LogicalSize::new(width, 350.0).into(),
    }
}

pub fn execute(app: &AppHandle, window: &Window) -> Result<serde_json::Value, String> {
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
    let main = app.get_webview("main").ok_or("MAIN_WEBVIEW_MISSING")?;
    let hwnd = window.hwnd().map_err(|_| "HWND_FAILED")?.0 as isize;
    let loads = Arc::new(AtomicUsize::new(0));
    let downloaded = Arc::new(AtomicBool::new(false));
    let create = |slot: usize, step: usize| -> Result<Arc<AtomicBool>, String> {
        let mut url = base.clone();
        let group = if slot.is_multiple_of(2) { "A" } else { "B" };
        url.query_pairs_mut()
            .append_pair("slot", &slot.to_string())
            .append_pair("profile", group)
            .append_pair("step", &step.to_string());
        let origin = base.origin();
        let download_origin = origin.clone();
        let flag = Arc::new(AtomicBool::new(false));
        let loaded = flag.clone();
        let load_count = loads.clone();
        let download_flag = downloaded.clone();
        let destination = root.join("data/fixture-download.txt");
        let expected = destination.clone();
        on_ui(&main, move |platform| {
            let parent = Parent(NonZeroIsize::new(hwnd).ok_or("INVALID_HWND")?);
            let view = WebViewBuilder::new()
                .with_environment(platform.environment())
                .with_profile_name(format!("Workspace_{group}"))
                .with_bounds(bounds(760.0))
                .with_url(url.to_string())
                .with_navigation_handler(move |next| {
                    next.parse::<tauri::Url>()
                        .is_ok_and(|url| url.origin() == origin)
                })
                .with_on_page_load_handler(move |event, _| {
                    if matches!(event, wry::PageLoadEvent::Finished) {
                        load_count.fetch_add(1, Ordering::SeqCst);
                        loaded.store(true, Ordering::SeqCst);
                    }
                })
                .with_download_started_handler(move |url, path| {
                    if !url.parse::<tauri::Url>().is_ok_and(|url| {
                        url.origin() == download_origin && url.path() == "/download"
                    }) {
                        return false;
                    }
                    *path = destination.clone();
                    true
                })
                .with_download_completed_handler(move |_, path, success| {
                    download_flag.store(
                        success
                            && path.as_ref() == Some(&expected)
                            && std::fs::read(&expected)
                                .is_ok_and(|bytes| bytes == b"TOOLBOX-DOWNLOAD-FIXTURE"),
                        Ordering::SeqCst,
                    );
                })
                .build_as_child(&parent)
                .map_err(|_| "NAMED_CHILD_CREATE_FAILED")?;
            VIEWS.with_borrow_mut(|views| {
                if slot == views.len() {
                    views.push(Some(view));
                } else {
                    views[slot] = Some(view);
                }
            });
            Ok(())
        })?;
        wait_for(&flag)?;
        std::thread::sleep(Duration::from_millis(120));
        on_ui(&main, move |_| {
            VIEWS.with_borrow(|views| {
                views[slot]
                    .as_ref()
                    .ok_or("CHILD_MISSING")?
                    .set_visible(false)
                    .map_err(|_| "HIDE_FAILED".into())
            })
        })?;
        Ok(flag)
    };
    let mut flags = Vec::new();
    for slot in 0..count {
        flags.push(create(slot, 0)?);
    }
    for step in 1..=operations {
        let slot = (step - 1) % count;
        match (step - 1) % 4 {
            0 => on_ui(&main, move |_| {
                VIEWS.with_borrow(|views| {
                    for view in views.iter().flatten() {
                        view.set_visible(false).map_err(|_| "HIDE_FAILED")?;
                    }
                    let view = views[slot].as_ref().ok_or("CHILD_MISSING")?;
                    view.set_visible(true).map_err(|_| "SHOW_FAILED")?;
                    view.focus().map_err(|_| "FOCUS_FAILED".into())
                })
            })?,
            1 => {
                let width = if (step / 4).is_multiple_of(2) {
                    880.0
                } else {
                    800.0
                };
                window
                    .set_size(tauri::LogicalSize::new(width, 640.0))
                    .map_err(|_| "WINDOW_RESIZE_FAILED")?;
                on_ui(&main, move |_| {
                    VIEWS.with_borrow(|views| {
                        views[slot]
                            .as_ref()
                            .ok_or("CHILD_MISSING")?
                            .set_bounds(bounds(width - 40.0))
                            .map_err(|_| "RESIZE_FAILED".into())
                    })
                })?;
            }
            2 => {
                flags[slot].store(false, Ordering::SeqCst);
                on_ui(&main, move |_| {
                    VIEWS.with_borrow(|views| {
                        let view = views[slot].as_ref().ok_or("CHILD_MISSING")?;
                        let mut url: tauri::Url = view
                            .url()
                            .map_err(|_| "URL_FAILED")?
                            .parse()
                            .map_err(|_| "URL_INVALID")?;
                        url.query_pairs_mut()
                            .append_pair("cycle", &step.to_string());
                        view.load_url(url.as_str())
                            .map_err(|_| "NAVIGATE_FAILED".into())
                    })
                })?;
                wait_for(&flags[slot])?;
                std::thread::sleep(Duration::from_millis(120));
            }
            _ => {
                on_ui(&main, move |_| {
                    VIEWS.with_borrow_mut(|views| {
                        views[slot].take();
                    });
                    Ok(())
                })?;
                flags[slot] = create(slot, step)?;
            }
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    if operations > 0 {
        on_ui(&main, move |_| {
            VIEWS.with_borrow(|views| {
                let view = views[0].as_ref().ok_or("CHILD_MISSING")?;
                view.set_visible(true).map_err(|_| "SHOW_FAILED")?;
                view.evaluate_script("document.getElementById('download').click()")
                    .map_err(|_| "DOWNLOAD_TRIGGER_FAILED".into())
            })
        })?;
        wait_for(&downloaded)?;
    }
    let alive = on_ui(&main, |_| {
        Ok(VIEWS.with_borrow(|views| views.iter().flatten().count()))
    })?;
    if alive != count || app.windows().len() != 1 {
        return Err("CHILD_COUNT_INVALID".into());
    }
    let remaining = if operations > 0 {
        on_ui(&main, |_| {
            VIEWS.with_borrow_mut(|views| views.clear());
            Ok(0_usize)
        })?
    } else {
        if count > 0 {
            on_ui(&main, |_| {
                VIEWS.with_borrow(|views| {
                    views[0]
                        .as_ref()
                        .ok_or("CHILD_MISSING")?
                        .set_visible(true)
                        .map_err(|_| "SHOW_FAILED".into())
                })
            })?;
        }
        alive
    };
    Ok(
        serde_json::json!({"status":"PASS", "tabs":count, "operations":operations,
        "page_loads":loads.load(Ordering::SeqCst), "download_verified":downloaded.load(Ordering::SeqCst),
        "one_window":app.windows().len()==1, "attached_webviews":window.webviews().len(), "native_children":alive,
        "remaining_children":remaining, "closed_children":operations>0,
        "shared_environment":true, "environment_source":"same main PlatformWebview.environment() COM object passed to every Wry child",
        "profile_strategy":"shared Shell UDF + Workspace_A / Workspace_B named Profiles", "udf_relative":"data/webview/shell"}),
    )
}
