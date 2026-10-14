//! The user's preferences, kept in `preferences.json` in SAM's Colony config
//! directory.
//!
//! The Appearance and Accessibility fields mirror colony-ui's
//! `StandardPreferences` (Project-Colony-Resources, crates/colony-ui): the same
//! names, the same value strings and the same defaults, so the file reads like
//! every other Colony program's. SAM adds `language`.
//!
//! Loading never fails on a value: a field that is missing, of the wrong type
//! or from a newer build falls back to its default and the rest of the file is
//! kept. An unknown theme is kept as written and resolved to Gruvbox dark by
//! the UI, so a later build that knows it gets the user's choice back.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    Small,
    Default,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Small,
    Default,
    Large,
    XLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Fr,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Preferences {
    pub selected_theme: String,
    pub selected_variant: String,
    /// An accent key; "auto" is never stored here, that is `auto_accent`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_accent: Option<String>,
    pub auto_accent: bool,
    pub font_size: FontSize,
    pub animations: bool,
    pub high_contrast: bool,
    pub dyslexia_font: bool,
    pub reduce_motion: bool,
    pub text_size_a11y: TextSize,
    /// None follows the system language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            selected_theme: "gruvbox".to_string(),
            selected_variant: "dark".to_string(),
            selected_accent: None,
            auto_accent: false,
            font_size: FontSize::Default,
            animations: true,
            high_contrast: false,
            dyslexia_font: false,
            reduce_motion: false,
            text_size_a11y: TextSize::Default,
            language: None,
        }
    }
}

impl<'de> Deserialize<'de> for Preferences {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Each field on its own, laid over the defaults: a value that does not
        // parse is a value the user did not set, not a reason to drop the file.
        fn take<T: DeserializeOwned>(map: &Map<String, Value>, key: &str, slot: &mut T) {
            if let Some(value) = map.get(key).and_then(|v| T::deserialize(v).ok()) {
                *slot = value;
            }
        }

        let mut p = Preferences::default();
        let Value::Object(map) = Value::deserialize(deserializer)? else {
            return Ok(p);
        };
        take(&map, "selected_theme", &mut p.selected_theme);
        take(&map, "selected_variant", &mut p.selected_variant);
        take(&map, "selected_accent", &mut p.selected_accent);
        take(&map, "auto_accent", &mut p.auto_accent);
        take(&map, "font_size", &mut p.font_size);
        take(&map, "animations", &mut p.animations);
        take(&map, "high_contrast", &mut p.high_contrast);
        take(&map, "dyslexia_font", &mut p.dyslexia_font);
        take(&map, "reduce_motion", &mut p.reduce_motion);
        take(&map, "text_size_a11y", &mut p.text_size_a11y);
        take(&map, "language", &mut p.language);
        Ok(p)
    }
}

/// The preferences in `file`, or the defaults when it is missing or unreadable.
pub fn load(file: &Path) -> Preferences {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes `prefs` to `file` through a temporary file and a rename, so a crash
/// mid-write leaves the previous preferences intact.
pub fn save(file: &Path, prefs: &Preferences) -> io::Result<()> {
    let tmp = file.with_extension(format!("json.{}.tmp", std::process::id()));
    let json = serde_json::to_vec_pretty(prefs)?;
    std::fs::write(&tmp, json)
        .and_then(|()| std::fs::rename(&tmp, file))
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Preferences {
        serde_json::from_str(json).expect("lenient parsing never fails on a value")
    }

    #[test]
    fn an_empty_or_foreign_file_gives_the_defaults() {
        assert_eq!(parse("{}"), Preferences::default());
        assert_eq!(parse("[1, 2]"), Preferences::default());
        assert_eq!(parse("null"), Preferences::default());
    }

    #[test]
    fn a_bad_field_falls_back_alone_and_the_rest_is_kept() {
        let p = parse(
            r#"{ "font_size": "enormous", "high_contrast": "yes", "animations": false,
                 "text_size_a11y": "xlarge", "language": "de", "unknown": 1 }"#,
        );
        assert_eq!(p.font_size, FontSize::Default);
        assert!(!p.high_contrast);
        assert!(!p.animations);
        assert_eq!(p.text_size_a11y, TextSize::XLarge);
        assert_eq!(p.language, None);
    }

    #[test]
    fn an_unknown_theme_is_kept_for_the_ui_to_resolve() {
        let p = parse(r#"{ "selected_theme": "from-a-newer-build", "selected_variant": 3 }"#);
        assert_eq!(p.selected_theme, "from-a-newer-build");
        assert_eq!(p.selected_variant, "dark");
    }

    #[test]
    fn the_file_uses_the_colony_field_names_and_round_trips() {
        let p = Preferences {
            selected_theme: "nord".into(),
            selected_variant: "light".into(),
            selected_accent: Some("violet".into()),
            font_size: FontSize::Large,
            text_size_a11y: TextSize::XLarge,
            language: Some(Language::Fr),
            ..Preferences::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["selected_accent"], "violet");
        assert_eq!(json["font_size"], "large");
        assert_eq!(json["text_size_a11y"], "xlarge");
        assert_eq!(json["language"], "fr");
        assert_eq!(serde_json::from_value::<Preferences>(json).unwrap(), p);

        // No accent and the system language are omitted, never written as null.
        let json = serde_json::to_value(Preferences::default()).unwrap();
        assert!(json.get("selected_accent").is_none());
        assert!(json.get("language").is_none());
    }

    #[test]
    fn save_then_load_round_trips_and_leaves_no_temporary_file() {
        let dir = std::env::temp_dir().join(format!("sam-preferences-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("preferences.json");

        assert_eq!(load(&file), Preferences::default());
        let p = Preferences { high_contrast: true, ..Preferences::default() };
        save(&file, &p).unwrap();
        assert_eq!(load(&file), p);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
