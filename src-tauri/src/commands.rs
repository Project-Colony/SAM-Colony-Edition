//! The Tauri command handlers the frontend calls through `invoke`.

use crate::dataset::{self, Game};
use crate::error::UiError;
use crate::paths;
use crate::preferences::{self, Preferences};
use crate::state::{AppState, ServiceAccess};
use crate::steam::{self, Achievement, Stat, User};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::PoisonError;
use steamworks::Client;
use tauri::{AppHandle, Manager, State};

/// Runs `op` on the blocking thread pool with the Steam client locked.
///
/// Steam IPC can stall, and `start_client` and `store_stats` wait up to 5 s
/// for Steam's answer, so none of it may run on the main thread, which also
/// paints the window. Holding the client lock for the whole operation keeps
/// Steam calls one at a time, as they were when every command ran on the main
/// thread: a game switch never overlaps a load or a write. No main-thread
/// command takes this lock, so waiting on it never freezes the window.
async fn with_steam<T, F>(app_handle: AppHandle, op: F) -> Result<T, UiError>
where
    T: Send + 'static,
    F: FnOnce(&mut Option<Client>) -> Result<T, UiError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let state: State<AppState> = app_handle.state();
        // A panic inside a Steam call must not lock every later command out.
        let mut client = state.client.lock().unwrap_or_else(PoisonError::into_inner);
        op(&mut client)
    })
    .await
    .map_err(|e| UiError::with("error_steam_call_failed", e))?
}

fn loaded(client: &Option<Client>) -> Result<Client, UiError> {
    client
        .clone()
        .ok_or_else(|| UiError::new("error_no_game_loaded"))
}

#[tauri::command]
pub async fn cmd_fetch_games(app_handle: AppHandle) -> Result<&'static str, UiError> {
    // Up to 15 s of network: keep it off the main thread.
    let (games, status) = tauri::async_runtime::spawn_blocking(|| {
        dataset::fetch_games().map_err(|e| UiError::with("error_load_database", e))
    })
    .await
    .map_err(|e| UiError::with("error_load_database", e))??;
    let state: State<AppState> = app_handle.state();
    *state.data.lock().unwrap() = Some(games);
    Ok(status)
}

#[tauri::command]
pub fn cmd_request_app_name(handle: AppHandle, appid: u32) -> String {
    handle.data(|games| {
        games.iter()
            .find(|g| g.appid == appid)
            .map(|g| g.name.clone())
            .unwrap_or_default()
    })
}

#[tauri::command]
pub fn cmd_search_name(handle: AppHandle, query: String) -> Vec<Game> {
    handle.data(|games| {
        dataset::fuzzy_search(games, &query, 10)
    })
}

#[tauri::command]
pub async fn cmd_start_client(app_handle: AppHandle, appid: u32) -> Result<(), UiError> {
    with_steam(app_handle, move |client| {
        // Shut the previous game's session down before starting the next one.
        *client = None;
        *client = Some(steam::start_client(appid)?);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn cmd_retrieve_user(app_handle: AppHandle) -> Result<User, UiError> {
    with_steam(app_handle, |client| {
        Ok(client.clone().map(steam::retrieve_user).unwrap_or_default())
    })
    .await
}

#[tauri::command]
pub async fn cmd_load_achievements(app_handle: AppHandle) -> Result<Vec<Achievement>, UiError> {
    with_steam(app_handle, |client| match client.clone() {
        Some(client) => steam::load_achievements(client),
        None => Ok(Vec::new()),
    })
    .await
}

#[tauri::command]
pub async fn cmd_load_achievement_icons(
    app_handle: AppHandle,
    appid: u32,
) -> Result<HashMap<String, String>, UiError> {
    with_steam(app_handle, move |client| {
        Ok(match client {
            Some(_) => steam::load_achievement_icons(appid),
            None => HashMap::new(),
        })
    })
    .await
}

#[tauri::command]
pub async fn cmd_commit_achievement(
    app_handle: AppHandle,
    name: String,
    unlocked: bool,
) -> Result<(), UiError> {
    with_steam(app_handle, move |client| {
        steam::commit_achievement(loaded(client)?, name, unlocked)
    })
    .await
}

#[tauri::command]
pub async fn cmd_store_stats(app_handle: AppHandle) -> Result<(), UiError> {
    with_steam(app_handle, |client| steam::store_stats(loaded(client)?)).await
}

#[tauri::command]
pub async fn cmd_load_statistics(app_handle: AppHandle, appid: u32) -> Result<Vec<Stat>, UiError> {
    with_steam(app_handle, move |client| {
        Ok(match client.clone() {
            Some(client) => steam::load_statistics(client, appid),
            None => Vec::new(),
        })
    })
    .await
}

#[tauri::command]
pub async fn cmd_commit_statistics(app_handle: AppHandle, name: String, value: i32) -> Result<(), UiError> {
    with_steam(app_handle, move |client| {
        steam::commit_statistics(loaded(client)?, name, value)
    })
    .await
}

fn preferences_file() -> std::io::Result<PathBuf> {
    Ok(paths::config_dir(paths::PROGRAM)?.join("preferences.json"))
}

#[tauri::command]
pub fn cmd_load_preferences() -> Preferences {
    preferences_file()
        .map(|file| preferences::load(&file))
        .unwrap_or_default()
}

#[tauri::command]
pub fn cmd_save_preferences(preferences: Preferences) -> Result<(), UiError> {
    preferences_file()
        .and_then(|file| preferences::save(&file, &preferences))
        .map_err(|e| UiError::with("error_save_preferences", e))
}

/// Where SAM keeps its files, for the About page. Nothing is created.
#[derive(Serialize)]
pub struct StoragePaths {
    config: Option<PathBuf>,
    cache: Option<PathBuf>,
}

#[tauri::command]
pub fn cmd_storage_paths() -> StoragePaths {
    StoragePaths {
        config: paths::locate::config_dir(paths::PROGRAM).ok(),
        cache: paths::locate::cache_dir(paths::PROGRAM).ok(),
    }
}
