// Rumba desktop: a minimal native window around https://rumba.hesap.net.tr using the system
// WebView (WebView2 on Windows), so it stays tiny and light on memory.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

const APP_URL: &str = "https://rumba.hesap.net.tr/";
// A normal desktop Edge user agent so sites — including Google sign-in — treat this window as a
// regular browser rather than an embedded webview (Google blocks OAuth in embedded user-agents).
const DESKTOP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0";

// Rumba itself and the whole Google sign-in flow resolve inside the app window.
fn stays_in_app(url: &tauri::Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    host == "rumba.hesap.net.tr"
        || host == "google.com"
        || host.ends_with(".google.com")
        || host.ends_with(".gstatic.com")
        || host.ends_with(".googleusercontent.com")
        || host == "accounts.youtube.com"
}

// Window controls and dragging are driven from the web UI via these commands, which call the
// native window directly — more reliable for remotely-hosted content than the JS window API.
#[tauri::command]
fn win_min(window: tauri::Window) { let _ = window.minimize(); }
#[tauri::command]
fn win_close(window: tauri::Window) { let _ = window.close(); }
#[tauri::command]
fn win_toggle(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) { let _ = window.unmaximize(); } else { let _ = window.maximize(); }
}
#[tauri::command]
fn win_drag(window: tauri::Window) { let _ = window.start_dragging(); }

fn main() {
    tauri::Builder::default()
        // Starting Rumba again just focuses the existing window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![win_min, win_close, win_toggle, win_drag])
        .setup(|app| {
            let handle = app.handle().clone();
            let builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(APP_URL.parse().unwrap()))
                .title("Rumba")
                .inner_size(1280.0, 800.0)
                .min_inner_size(940.0, 600.0)
                .center()
                .user_agent(DESKTOP_UA)
                .background_color(tauri::window::Color(30, 31, 34, 255))
                // Tell the web UI which OS it runs on (the UA is spoofed to Edge for Google), and
                // that this build can open URLs in the system browser (Google sign-in, message links).
                .initialization_script(&format!("window.__RUMBA_OS__={:?};window.__RUMBA_SYSBROWSER__=true;", std::env::consts::OS))
                // Keep Rumba + Google auth in-app; send any other site to the system browser.
                .on_navigation(move |url| {
                    if stays_in_app(url) {
                        return true;
                    }
                    match url.scheme() {
                        "data" | "about" | "blob" => true,
                        _ => {
                            let _ = handle.opener().open_url(url.as_str(), None::<&str>);
                            false
                        }
                    }
                });

            // A dark, app-coloured title bar: transparent overlay on macOS (native traffic
            // lights kept), frameless on Windows (the web UI draws its own window controls).
            #[cfg(target_os = "macos")]
            let builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
            #[cfg(target_os = "windows")]
            let builder = builder.decorations(false);

            builder.build()?;

            // Game detection (Discord-style): the web UI sends the list of game executables to watch
            // (from the server's games.json); we poll running processes and report the active game's
            // exe back to the web UI, which reflects it in the user's status.
            let games: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let games_listen = games.clone();
            app.listen_any("rumba:games", move |event| {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    if let Ok(mut g) = games_listen.lock() {
                        *g = list.into_iter().map(|s| s.to_lowercase()).collect();
                    }
                }
            });
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut sys = System::new();
                let mut last = String::new();
                loop {
                    std::thread::sleep(Duration::from_secs(20));
                    let watch = match games.lock() {
                        Ok(g) => g.clone(),
                        Err(_) => continue,
                    };
                    let mut found = String::new();
                    if !watch.is_empty() {
                        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
                        for proc_ in sys.processes().values() {
                            let name = proc_.name().to_string_lossy().to_lowercase();
                            if watch.iter().any(|w| *w == name) {
                                found = name;
                                break;
                            }
                        }
                    }
                    if found != last {
                        last = found.clone();
                        if let Some(w) = handle.get_webview_window("main") {
                            let arg = serde_json::to_string(&found).unwrap_or_else(|_| "\"\"".into());
                            let _ = w.eval(&format!("window.__RUMBA_GAME__&&window.__RUMBA_GAME__({arg})"));
                        }
                    }
                }
            });

            // Auto-update (Windows): check the server manifest on launch and, if a newer signed
            // build exists, download, install and relaunch silently. No code-signing cert needed —
            // updates are verified against the embedded minisign public key.
            #[cfg(target_os = "windows")]
            {
                use tauri_plugin_updater::UpdaterExt;
                let updater_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Ok(updater) = updater_handle.updater() {
                        if let Ok(Some(update)) = updater.check().await {
                            if update.download_and_install(|_, _| {}, || {}).await.is_ok() {
                                updater_handle.restart();
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Rumba başlatılamadı");
}
