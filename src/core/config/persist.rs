//! Persisted view selection shared by the CLI and the TUI.
//!
//! The CLI writes `config.json`; the TUI polls this view at tick boundaries
//! so `--change-theme` and `layout set` reach running instances without IPC.
//! A missing or invalid file is ignored so a partial write never resets the
//! live selection.

use std::fs;

use crate::core::config;

/// Theme and layout persisted in `config.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedSelection {
    pub theme: String,
    /// Display name of the selected layout; empty when the mode is used.
    pub layout: String,
}

/// Read the persisted selection, or `None` when the file is missing or does
/// not parse.
pub fn persisted_selection() -> Option<PersistedSelection> {
    let data = fs::read_to_string(config::config_path()).ok()?;
    let cfg: config::Config = serde_json::from_str(&data).ok()?;
    Some(PersistedSelection {
        theme: cfg.theme,
        layout: cfg.layout_name,
    })
}

/// Theme persisted in the user config.
pub fn persisted_theme() -> Option<String> {
    persisted_selection().map(|selection| selection.theme)
}

/// Layout display name persisted in the user config, if one is set.
pub fn persisted_layout() -> Option<String> {
    persisted_selection()
        .map(|selection| selection.layout)
        .filter(|layout| !layout.is_empty())
}
