use reqwest::blocking::Client;
use std::time::Duration;
use std::path::{Path, PathBuf};
use crate::paths;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub appid: u32,
    pub name: String,
}

/// The games list and a status code for the UI saying where it came from.
pub fn fetch_games() -> Result<(Vec<Game>, &'static str), Box<dyn std::error::Error>> {
    let cache_path = get_cache_path()?;

    if cache_is_fresh(&cache_path) {
        let data = std::fs::read_to_string(&cache_path)?;
        let games = serde_json::from_str(&data)?;
        return Ok((games, "status_database_cached"));
    }

    match fetch_network() {
        Ok(games) => {
            let _ = write_cache(&cache_path, &games);
            Ok((games, "status_database_updated"))
        }
        Err(e) => {
            if cache_path.exists() {
                let data = std::fs::read_to_string(&cache_path)?;
                let games = serde_json::from_str(&data)?;
                Ok((games, "status_database_offline"))
            } else {
                Err(e)
            }
        }
    }
}

fn get_cache_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = paths::cache_dir(paths::PROGRAM)?.join("cache.json");
    let (Some(root), Ok(data)) = (dirs::cache_dir(), paths::data_dir(paths::PROGRAM)) else {
        return Ok(path);
    };
    Ok(migrate_legacy_cache(
        &root.join("SAM-Colony-Edition").join("cache.json"),
        &path,
        &data.join(MIGRATED_MARKER),
    ))
}

/// Written once the legacy games list has moved. It sits in the data
/// directory, not the cache, so clearing the cache does not bring back a
/// legacy copy that the move had to leave behind.
const MIGRATED_MARKER: &str = "legacy-cache-migrated";

/// Moves the games list from where releases before the Colony layout kept it,
/// `<cache root>/SAM-Colony-Edition/cache.json`, and returns the file to use.
///
/// It runs until it has moved a file: then `marker` is written and later
/// starts skip it. A new copy that already exists is never overwritten. A
/// rename keeps the file's age, so freshness is judged as before; if it fails
/// (another volume, a locked file), a copy dated like the original is the
/// fallback and the old file stays. If both fail, SAM keeps using the legacy
/// file, as filesystem.md requires of a failed migration.
// ponytail: the legacy SAM-Colony-Edition/ directory is never deleted here, as
// filesystem.md requires for the release that adds the move. Prune it in a
// later release.
fn migrate_legacy_cache(legacy: &Path, path: &Path, marker: &Path) -> PathBuf {
    if marker.exists() || path.exists() || !legacy.exists() {
        return path.to_path_buf();
    }
    let copy = || -> std::io::Result<()> {
        std::fs::copy(legacy, path)?;
        if let Ok(modified) = std::fs::metadata(legacy).and_then(|m| m.modified()) {
            let _ = std::fs::File::options()
                .write(true)
                .open(path)
                .and_then(|f| f.set_modified(modified));
        }
        Ok(())
    };
    if std::fs::rename(legacy, path).is_err() && copy().is_err() {
        let _ = std::fs::remove_file(path);
        return legacy.to_path_buf();
    }
    let _ = std::fs::write(marker, "");
    path.to_path_buf()
}

fn cache_is_fresh(path: &PathBuf) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|mtime| {
            SystemTime::now()
                .duration_since(mtime)
                .map(|age| age.as_secs() < 86400)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn fetch_network() -> Result<Vec<Game>, Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/jsnli/steamappidlist/master/data/games_appid.json";
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(concat!(
            "sam-colony-edition/",
            env!("CARGO_PKG_VERSION"),
            " (+https://github.com/Project-Colony/SAM-Colony-Edition)"
        ))
        .build()?;
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()).into());
    }
    Ok(resp.json()?)
}

fn write_cache(path: &PathBuf, games: &[Game]) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(games)?;
    std::fs::write(path, json)?;
    Ok(())
}

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub fn fuzzy_search(games: &[Game], query: &str, limit: usize) -> Vec<Game> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return vec![];
    }

    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<(i64, &Game)> = games
        .iter()
        .filter_map(|g| {
            let name_score = matcher.fuzzy_match(&g.name.to_lowercase(), &query);
            let id_score = matcher.fuzzy_match(&g.appid.to_string(), &query);
            name_score.or(id_score).map(|score| (score, g))
        })
        .collect();


    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));

    scored
        .into_iter()
        .take(limit)
        .map(|(_, g)| g.clone())
        .collect()

}

#[cfg(test)]
mod tests {
    #[test]
    fn the_legacy_cache_moves_once_and_never_over_a_new_one() {
        let dir = std::env::temp_dir().join(format!("sam-cache-migration-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("old")).unwrap();
        let (legacy, path, marker) =
            (dir.join("old/cache.json"), dir.join("cache.json"), dir.join("marker"));

        // Nothing to move: the new path, and no marker, so a later start looks again.
        assert_eq!(super::migrate_legacy_cache(&legacy, &path, &marker), path);
        assert!(!marker.exists());

        std::fs::write(&legacy, "[1]").unwrap();
        assert_eq!(super::migrate_legacy_cache(&legacy, &path, &marker), path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");
        assert!(!legacy.exists(), "a rename leaves no copy behind");
        assert!(marker.exists());

        // Once marked, a legacy file is left alone even with the cache cleared.
        std::fs::write(&legacy, "[2]").unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(super::migrate_legacy_cache(&legacy, &path, &marker), path);
        assert!(!path.exists());

        // An existing new cache is never overwritten by a legacy one.
        std::fs::remove_file(&marker).unwrap();
        std::fs::write(&path, "[1]").unwrap();
        assert_eq!(super::migrate_legacy_cache(&legacy, &path, &marker), path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_failed_move_keeps_the_legacy_cache() {
        let dir = std::env::temp_dir().join(format!("sam-cache-fallback-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("old")).unwrap();
        let legacy = dir.join("old/cache.json");
        std::fs::write(&legacy, "[1]").unwrap();
        // The new cache's directory does not exist, so neither rename nor copy works.
        let path = dir.join("missing/cache.json");
        let marker = dir.join("marker");

        assert_eq!(super::migrate_legacy_cache(&legacy, &path, &marker), legacy);
        assert!(legacy.exists() && !marker.exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
