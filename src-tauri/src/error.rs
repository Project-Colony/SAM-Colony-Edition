//! The error a command hands the UI: a stable code the UI translates
//! (src/i18n/*.json has a string for every code) and, when there is one, the
//! untranslated detail from Steam, the network or the OS.

use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Serialize)]
pub struct UiError {
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl UiError {
    pub fn new(code: &'static str) -> Self {
        UiError { code, detail: None }
    }

    pub fn with(code: &'static str, detail: impl Display) -> Self {
        UiError { code, detail: Some(detail.to_string()) }
    }
}
