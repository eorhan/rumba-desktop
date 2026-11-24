// Rumba desktop: a minimal native window around https://rumba.hesap.net.tr using the system
// WebView (WebView2 on Windows), so it stays tiny and light on memory.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

const APP_URL: &str = "https://rumba.hesap.net.tr/";
// Hosts that stay inside the app window (the app itself + Google sign-in flow).
const IN_APP_HOSTS: &[&str] = &["rumba.hesap.net.tr", "accounts.google.com", "accounts.youtube.com", "oauth2.googleapis.com"];

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
        .setup(|app| {
            let handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(APP_URL.parse().unwrap()))
                .title("Rumba")
                .inner_size(1280.0, 800.0)
                .min_inner_size(940.0, 600.0)
                .center()
                .background_color(tauri::window::Color(30, 31, 34, 255))
                // Links to other sites open in the default browser instead of inside the app.
                .on_navigation(move |url| {
                    let host = url.host_str().unwrap_or_default();
                    let inside = url.scheme() == "https" && IN_APP_HOSTS.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")));
                    if !inside && (url.scheme() == "https" || url.scheme() == "http") {
                        let _ = handle.opener().open_url(url.as_str(), None::<&str>);
                    }
                    inside || url.scheme() == "about" || url.scheme() == "data"
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Rumba başlatılamadı");
}
