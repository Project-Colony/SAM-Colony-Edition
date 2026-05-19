use crate::vdf::{self, VdfMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use steamworks::{Client, UserStatsReceived};

#[derive(Serialize, Deserialize)]
pub struct Achievement {
    pub api_name: String,
    pub name: String,
    pub desc: String,
    pub status: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Stat {
    pub api_name: String,
    pub name: String,
    pub min: i32,
    pub max: i32,
    pub value: i32,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    user_name: String,
    user_steam_id: u64,
}

impl Default for User {
    fn default() -> Self {
        User {
            user_name: "No user found.".to_string(),
            user_steam_id: 0,
        }
    }
}

pub fn start_client(appid: u32) -> Result<Client, String> {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let waiting = Arc::new(Mutex::new(true));
        let waiting_clone = Arc::clone(&waiting);

        let client = Client::init_app(appid).expect("init_app failed");

        let user_stats = client.user_stats();
        let steam_user_id: u64 = client.user().steam_id().raw();

        // IMPORTANT: bind the CallbackHandle to a named local. Dropping it
        // immediately unregisters the callback (steamworks 0.12 Drop impl).
        let _stats_cb = client.register_callback(move |_data: UserStatsReceived| {
            let mut waiting = waiting_clone.lock().unwrap();
            *waiting = false;
        });

        user_stats.request_user_stats(steam_user_id);
        client.run_callbacks();

        // Wait up to 5s for UserStatsReceived. Big games can need >1s on cold pipe.
        for _ in 0..50 {
            client.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
            if *waiting.lock().unwrap() == false {
                break;
            }
        }

        client
    }));

    match result {
        Ok(client) => Ok(client),
        Err(panic_error) => Err(format!("Panic occured: {:?}", panic_error)),
    }
}

pub fn retrieve_user(client: Client) -> User {
    User {
        user_name: client.friends().name(),
        user_steam_id: client.user().steam_id().raw(),
    }
}

pub fn load_achievements(client: Client) -> Result<Vec<Achievement>, String> {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        // Pump the callback queue once more in case UserStatsReceived
        // arrived after start_client returned.
        client.run_callbacks();

        let user_stats = client.user_stats();

        // Games with zero achievements make steamworks-rs panic inside
        // get_achievement_names (it calls .expect on get_num_achievements).
        // Bail out cleanly before we ever hit that.
        if user_stats.get_num_achievements().unwrap_or(0) == 0 {
            return Vec::new();
        }

        let names = user_stats
            .get_achievement_names()
            .unwrap_or_default();
        let mut list: Vec<Achievement> = Vec::with_capacity(names.len());
        for name in names {
            // Defensive: skip names containing null bytes (CString::new panics on them).
            if name.contains('\0') {
                continue;
            }
            let helper = user_stats.achievement(&name);
            let display_name = helper
                .get_achievement_display_attribute("name")
                .map(|s| s.to_string())
                .unwrap_or_else(|_| name.clone());
            let desc = helper
                .get_achievement_display_attribute("desc")
                .map(|s| s.to_string())
                .unwrap_or_default();
            let status = helper.get().unwrap_or(false);
            list.push(Achievement {
                api_name: name,
                name: display_name,
                desc,
                status,
            });
        }

        list
    }));

    match result {
        Ok(list) => Ok(list),
        Err(panic_error) => Err(format!("Panic occured: {:?}", panic_error)),
    }
}

pub fn load_achievement_icons(appid: u32) -> HashMap<String, String> {
    let mut paths: HashMap<String, String> = HashMap::new();

    let game_root = match load_schema(appid) {
        Ok(r) => r,
        Err(e) => { eprintln!("load_schema error: {}", e); return paths; }
    };

    let stats_map = match game_root.get("stats").and_then(|v| v.as_map()) {
        Some(m) => m,
        None => return paths,
    };

    for entry in stats_map.values().filter_map(|v| v.as_map()) {
        if entry.get("type").and_then(|v| v.as_str()) != Some("ACHIEVEMENTS") {
            continue;
        }
        let bits = match entry.get("bits").and_then(|v| v.as_map()) {
            Some(b) => b,
            None => continue,
        };
        for bit in bits.values().filter_map(|v| v.as_map()) {
            let api_name = match bit.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let display = match bit.get("display").and_then(|v| v.as_map()) {
                Some(d) => d,
                None => continue,
            };
            if let Some(icon) = display.get("icon").and_then(|v| v.as_str()) {
                paths.insert(api_name.clone(), format!(
                    "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{}/{}",
                    appid, icon
                ));
            }
            if let Some(icon_gray) = display.get("icon_gray").and_then(|v| v.as_str()) {
                paths.insert(api_name + "-gray", format!(
                    "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{}/{}",
                    appid, icon_gray
                ));
            }
        }
        // Steam splits >32 achievements across multiple ACHIEVEMENTS blocks
        // (each block is a 32-bit bitfield). Do NOT break - iterate them all.
    }

    paths
}

pub fn commit_achievement(client: Client, name: String, unlocked: bool) {
    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(&name);
    if unlocked {
        let _ = achievement.set();
    } else {
        let _ = achievement.clear();
    }
}

pub fn store_stats(client: Client) {
    let user_stats = client.user_stats();
    let _ = user_stats.store_stats();
}

/// Returns the first existing Steam install root for the current OS, or None.
///
/// - Windows: `HKCU\Software\Valve\Steam!SteamPath` (registry) then the
///   common `C:\Program Files (x86)\Steam` fallback.
/// - macOS: `~/Library/Application Support/Steam`.
/// - Linux: `~/.steam/steam` then `~/.local/share/Steam` (Flatpak-free
///   installs use one or the other depending on distro).
pub fn steam_root() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(p) = windows_steam_root() {
            if p.exists() { return Some(p); }
        }
        let fallback = PathBuf::from(r"C:\Program Files (x86)\Steam");
        if fallback.exists() { return Some(fallback); }
        None
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        let p = PathBuf::from(home).join("Library/Application Support/Steam");
        if p.exists() { Some(p) } else { None }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let home = std::env::var_os("HOME")?;
        let home = PathBuf::from(home);
        for rel in [".steam/steam", ".local/share/Steam"] {
            let p = home.join(rel);
            if p.exists() { return Some(p); }
        }
        None
    }
}

#[cfg(target_os = "windows")]
fn windows_steam_root() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Valve\\Steam").ok()?;
    let path: String = key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(path))
}

pub fn load_schema(appid: u32) -> std::io::Result<VdfMap> {
    let root = steam_root().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Steam install not found (set STEAM_ROOT env var to override)",
        )
    })?;
    let path = root
        .join("appcache")
        .join("stats")
        .join(format!("UserGameStatsSchema_{}.bin", appid));
    let bytes = std::fs::read(&path)?;
    let mut parser = vdf::Parser::new(&bytes);
    let mut root_map = parser.parse_object()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let game_root = root_map.values_mut().next()
        .and_then(|v| if let vdf::VdfValue::Nested(m) = v { Some(std::mem::take(m)) } else { None })
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no game root"))?;
    Ok(game_root)
}

pub fn load_statistics(client: Client, appid: u32) -> Vec<Stat> {
    let user_stats = client.user_stats();

    let game_root = match load_schema(appid) {
        Ok(r) => r,
        Err(e) => { println!("{}", e); return Vec::new(); }
    };

    let stats_map = match game_root.get("stats").and_then(|v| v.as_map()) {
        Some(m) => m,
        None => return Vec::new(),
    };

    stats_map.values()
        .filter_map(|v| v.as_map())
        .filter(|m| m.get("type").and_then(|v| v.as_str()) == Some("INT"))
        .map(|m| {
            let api_name = m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let display_name = m.get("display")
                .and_then(|v| v.as_map())
                .and_then(|d| d.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let max = m.get("max").and_then(|v| v.as_int()).unwrap_or(0);
            let value = user_stats.get_stat_i32(&api_name).unwrap_or(0);
            Stat { api_name, name: display_name, min: 0, max, value }
        })
        .collect()
}

pub fn commit_statistics(client: Client, name: String, value: i32) {
    let user_stats = client.user_stats();
    let _ = user_stats.set_stat_i32(&name, value);
}
