//! Persist runtime-owned state into the user config file.

use crate::core::config;

use super::AppState;

/// Persist the runtime state into the user config file.
///
/// Existing user values (`history_points`, update interval when untouched,
/// keybindings, alerts) are preserved: only the fields the runtime owns are
/// overwritten.
pub fn save_config(state: &AppState) {
    let mut cfg = config::load_config();
    cfg.theme = state.current_theme.name.clone();
    if let Some(def) = state.layout_defs.get(state.layout_index) {
        cfg.layout_name = def.name.clone();
    }
    cfg.layout_mode = state.layout_mode;
    cfg.update_interval_ms = state.update_interval_ms;
    cfg.alerts = state.alerts.clone();
    cfg.keybindings = state.keybindings.clone();
    let _ = config::save_config(&cfg);
}
