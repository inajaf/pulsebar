mod commands;
mod notify;
mod sensors;
mod state;
mod tray;

use state::{AppState, MetricsSnapshot};
use std::time::Duration;
use sysinfo::System;
use tauri::{Emitter, Manager, WindowEvent};

const METRICS_EVENT: &str = "metrics://update";
const VISIBLE_POLL_INTERVAL: Duration = Duration::from_secs(1);
const HIDDEN_POLL_INTERVAL: Duration = Duration::from_secs(3);
const MAIN_WINDOW: &str = "main";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch should surface the existing tray app, not start another.
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![commands::get_current_metrics])
        .setup(|app| {
            let handle = app.handle().clone();

            tray::build(&handle)?;

            #[cfg(target_os = "macos")]
            handle.set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            if let Some(window) = handle.get_webview_window(MAIN_WINDOW) {
                #[cfg(not(target_os = "macos"))]
                let _ = window.set_skip_taskbar(true);

                // Keep the tray app (and its background poller) alive when the
                // user clicks the window's close button — hide instead of destroy.
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
                    }
                });
            }

            spawn_metrics_loop(handle.clone());
            spawn_disk_scan_loop(handle);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn spawn_metrics_loop(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new_all();
        let mut tick: u64 = 0;

        loop {
            let visible = app
                .get_webview_window(MAIN_WINDOW)
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);

            // The process walk dominates the tick cost; only pay for it while
            // someone can see the lists, and even then every other second.
            let include_processes = visible && tick % 2 == 0;
            let mut snapshot = sensors::sample(&mut sys, include_processes);

            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(disk_top) = state.disk_top.lock() {
                    snapshot.top_disk = disk_top.clone();
                }
                if let Ok(mut guard) = state.snapshot.lock() {
                    if !include_processes {
                        snapshot.top_cpu = guard.top_cpu.clone();
                        snapshot.top_mem = guard.top_mem.clone();
                    }
                    *guard = snapshot.clone();
                }
            }

            let _ = app.emit(METRICS_EVENT, &snapshot);
            tray::update_tooltip(&app, &format_tooltip(&snapshot));
            notify::check_and_notify(&app, &snapshot);

            let interval = if visible {
                VISIBLE_POLL_INTERVAL
            } else {
                HIDDEN_POLL_INTERVAL
            };
            tick = tick.wrapping_add(1);
            tokio::time::sleep(interval).await;
        }
    });
}

/// Walking /Applications takes seconds and its result changes rarely, so it
/// runs on its own slow loop in a blocking thread, far away from the 1s tick.
fn spawn_disk_scan_loop(app: tauri::AppHandle) {
    const SCAN_INTERVAL: Duration = Duration::from_secs(600);

    tauri::async_runtime::spawn(async move {
        loop {
            let top = tauri::async_runtime::spawn_blocking(|| sensors::storage::top_apps(3))
                .await
                .unwrap_or_default();

            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(mut guard) = state.disk_top.lock() {
                    *guard = top;
                }
            }
            tokio::time::sleep(SCAN_INTERVAL).await;
        }
    });
}

fn format_tooltip(snapshot: &MetricsSnapshot) -> String {
    let fmt = |p: Option<f32>| p.map(|v| format!("{v:.0}%")).unwrap_or_else(|| "--".into());
    format!(
        "Pulsebar\nCPU {}  ·  RAM {}  ·  GPU {}",
        fmt(snapshot.cpu.percent),
        fmt(snapshot.ram.percent),
        fmt(snapshot.gpu.percent)
    )
}
