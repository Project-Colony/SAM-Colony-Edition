// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod bootstrap;
mod commands;
mod dataset;
mod paths;
mod state;
mod steam;
mod vdf;

use state::AppState;
use std::sync::Mutex;

fn main() {
    // MUST run before any code that might trigger Steamworks DLL/dylib
    // resolution. See bootstrap.rs for the full rationale.
    bootstrap::bootstrap();

    tauri::Builder::default()
        .manage(AppState {
            data: Mutex::new(None),
            client: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::cmd_fetch_games,
            commands::cmd_request_app_name,
            commands::cmd_search_name,
            commands::cmd_start_client,
            commands::cmd_load_achievements,
            commands::cmd_load_achievement_icons,
            commands::cmd_commit_achievement,
            commands::cmd_store_stats,
            commands::cmd_load_statistics,
            commands::cmd_commit_statistics,
            commands::cmd_retrieve_user,
        ])
        .setup(|app| {
            // Built here rather than in tauri.conf.json so the WebView keeps its
            // data under Colony/SAM instead of a directory named after the
            // bundle identifier. WKWebView ignores data_directory, so on macOS
            // it stays where the system puts it.
            let mut window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
                .title("SAM - Colony Edition")
                .inner_size(1280.0, 720.0)
                .min_inner_size(960.0, 600.0)
                .use_https_scheme(true);
            match paths::data_dir(paths::PROGRAM) {
                Ok(dir) => window = window.data_directory(dir.join("webview")),
                Err(e) => eprintln!("[setup] no data directory, the WebView uses its default: {e}"),
            }
            window.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
