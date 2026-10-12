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
use std::ffi::OsString;
use std::sync::Mutex;

fn main() {
    // Answered before anything else, Steam included: the release workflow
    // runs `--version` on every binary to prove it starts at all.
    if is_version_request(std::env::args_os()) {
        print_version();
        return;
    }

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

/// True when the first argument after the program name is `--version`.
fn is_version_request(mut args: impl Iterator<Item = OsString>) -> bool {
    args.nth(1).is_some_and(|arg| arg == "--version")
}

fn print_version() {
    // A release build is a GUI-subsystem program with no console of its own.
    // Borrow the parent's, so the line reaches the terminal that ran it.
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    println!("sam-colony-edition {}", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    use super::is_version_request;
    use std::ffi::OsString;

    fn args(list: &[&str]) -> impl Iterator<Item = OsString> {
        list.iter().map(OsString::from).collect::<Vec<_>>().into_iter()
    }

    #[test]
    fn only_a_leading_version_flag_is_a_version_request() {
        assert!(is_version_request(args(&["samira", "--version"])));
        assert!(!is_version_request(args(&["samira"])));
        assert!(!is_version_request(args(&["samira", "-V"])));
        assert!(!is_version_request(args(&["samira", "--debug", "--version"])));
    }
}
