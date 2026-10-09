// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod bootstrap;
mod commands;
mod dataset;
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
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
