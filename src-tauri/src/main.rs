// Rumba desktop: a minimal native window around https://rumba.hesap.net.tr using the system
// WebView (WebView2 on Windows), so it stays tiny and light on memory.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
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
                .user_agent(DESKTOP_UA)
                .background_color(tauri::window::Color(30, 31, 34, 255))
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
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Rumba başlatılamadı");
}
