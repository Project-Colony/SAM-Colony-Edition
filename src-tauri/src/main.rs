// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod bootstrap;
mod dataset;
mod state;
mod steam;
mod vdf;

use dataset::Game;
use state::{AppState, ServiceAccess};
use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use steam::{Achievement, Stat, User};
use steamworks::Client;
use tauri::{AppHandle, Manager, State};

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
            cmd_fetch_games,
            cmd_request_app_name,
            cmd_search_name,
            cmd_start_client,
            cmd_load_achievements,
            cmd_load_achievement_icons,
            cmd_commit_achievement,
            cmd_store_stats,
            cmd_load_statistics,
            cmd_commit_statistics,
            cmd_retrieve_user,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Runs `op` on the blocking thread pool with the Steam client locked.
///
/// Steam IPC can stall, and `start_client` and `store_stats` wait up to 5 s
/// for Steam's answer, so none of it may run on the main thread, which also
/// paints the window. Holding the client lock for the whole operation keeps
/// Steam calls one at a time, as they were when every command ran on the main
/// thread: a game switch never overlaps a load or a write. No main-thread
/// command takes this lock, so waiting on it never freezes the window.
async fn with_steam<T, F>(app_handle: AppHandle, op: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Option<Client>) -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let state: State<AppState> = app_handle.state();
        // A panic inside a Steam call must not lock every later command out.
        let mut client = state.client.lock().unwrap_or_else(PoisonError::into_inner);
        op(&mut client)
    })
    .await
    .map_err(|e| format!("Steam call failed: {e}"))?
}

fn loaded(client: &Option<Client>) -> Result<Client, String> {
    client
        .clone()
        .ok_or_else(|| "No game is loaded. Pick a game first.".to_string())
}

#[tauri::command]
async fn cmd_fetch_games(app_handle: AppHandle) -> Result<String, String> {
    // Up to 15 s of network: keep it off the main thread.
    let (games, status) = tauri::async_runtime::spawn_blocking(|| {
        dataset::fetch_games().map_err(|e| format!("Failed to load database: {e}"))
    })
    .await
    .map_err(|e| format!("Failed to load database: {e}"))??;
    let state: State<AppState> = app_handle.state();
    *state.data.lock().unwrap() = Some(games);
    Ok(status)
}

#[tauri::command]
fn cmd_request_app_name(handle: AppHandle, appid: u32) -> String {
    handle.data(|games| {
        games.iter()
            .find(|g| g.appid == appid)
            .map(|g| g.name.clone())
            .unwrap_or_default()
    })
}

#[tauri::command]
fn cmd_search_name(handle: AppHandle, query: String) -> Vec<Game> {
    handle.data(|games| {
        dataset::fuzzy_search(games, &query, 10)
    })
}

#[tauri::command]
async fn cmd_start_client(app_handle: AppHandle, appid: u32) -> Result<(), String> {
    with_steam(app_handle, move |client| {
        // Shut the previous game's session down before starting the next one.
        *client = None;
        *client = Some(steam::start_client(appid)?);
        Ok(())
    })
    .await
}

#[tauri::command]
async fn cmd_retrieve_user(app_handle: AppHandle) -> Result<User, String> {
    with_steam(app_handle, |client| {
        Ok(client.clone().map(steam::retrieve_user).unwrap_or_default())
    })
    .await
}

#[tauri::command]
async fn cmd_load_achievements(app_handle: AppHandle) -> Result<Vec<Achievement>, String> {
    with_steam(app_handle, |client| match client.clone() {
        Some(client) => steam::load_achievements(client),
        None => Ok(Vec::new()),
    })
    .await
}

#[tauri::command]
async fn cmd_load_achievement_icons(
    app_handle: AppHandle,
    appid: u32,
) -> Result<HashMap<String, String>, String> {
    with_steam(app_handle, move |client| {
        Ok(match client {
            Some(_) => steam::load_achievement_icons(appid),
            None => HashMap::new(),
        })
    })
    .await
}

#[tauri::command]
async fn cmd_commit_achievement(
    app_handle: AppHandle,
    name: String,
    unlocked: bool,
) -> Result<(), String> {
    with_steam(app_handle, move |client| {
        steam::commit_achievement(loaded(client)?, name, unlocked)
    })
    .await
}

#[tauri::command]
async fn cmd_store_stats(app_handle: AppHandle) -> Result<(), String> {
    with_steam(app_handle, |client| steam::store_stats(loaded(client)?)).await
}

#[tauri::command]
async fn cmd_load_statistics(app_handle: AppHandle, appid: u32) -> Result<Vec<Stat>, String> {
    with_steam(app_handle, move |client| {
        Ok(match client.clone() {
            Some(client) => steam::load_statistics(client, appid),
            None => Vec::new(),
        })
    })
    .await
}

#[tauri::command]
async fn cmd_commit_statistics(app_handle: AppHandle, name: String, value: i32) -> Result<(), String> {
    with_steam(app_handle, move |client| {
        steam::commit_statistics(loaded(client)?, name, value)
    })
    .await
}
