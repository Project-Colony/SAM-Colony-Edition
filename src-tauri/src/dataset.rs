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
    if let Some(root) = dirs::cache_dir() {
        migrate_legacy_cache(&root.join("SAM-Colony-Edition").join("cache.json"), &path);
    }
    Ok(path)
}

/// Moves the games list from where releases before the Colony layout kept it,
/// `<cache root>/SAM-Colony-Edition/cache.json`, when the new copy is missing.
///
/// A rename keeps the file's age, so freshness is judged as before. If it
/// fails (another volume, a locked file), the copy is the fallback and the old
/// file stays; a failed copy just means the list is downloaded again.
// ponytail: the legacy SAM-Colony-Edition/ directory is never deleted here, as
// filesystem.md requires for the release that adds the move. Prune it in a
// later release.
fn migrate_legacy_cache(legacy: &Path, path: &Path) {
    if !path.exists() && legacy.exists() && std::fs::rename(legacy, path).is_err() {
        let _ = std::fs::copy(legacy, path);
    }
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


    scored.sort_by(|a, b| b.0.cmp(&a.0));

    scored
        .into_iter()
        .take(limit)
        .map(|(_, g)| g.clone())
        .collect()

}

#[cfg(test)]
mod tests {
    #[test]
    fn the_legacy_cache_moves_only_when_the_new_one_is_missing() {
        let dir = std::env::temp_dir().join(format!("sam-cache-migration-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("old")).unwrap();
        let (legacy, path) = (dir.join("old/cache.json"), dir.join("cache.json"));

        std::fs::write(&legacy, "[1]").unwrap();
        super::migrate_legacy_cache(&legacy, &path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");
        assert!(!legacy.exists(), "a rename leaves no copy behind");

        // An existing new cache is never overwritten by a legacy one.
        std::fs::write(&legacy, "[2]").unwrap();
        super::migrate_legacy_cache(&legacy, &path);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[1]");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
