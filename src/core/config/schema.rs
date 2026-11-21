//! Persisted configuration schema.
//!
//! The on-disk config of the app. Theme and layout names are plain strings
//! here; the schema types stay independent of the runtime state.

use crate::core::config::keybinding::Keybindings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xtop_layout::LayoutMode;
// Glyph style enums are shared ecosystem-wide (kernel + widget packs).
pub use xtop_widget_api::{ChartCharset, WidgetBorders};
// Alert thresholds are contract data (DR-1): the plugin-api type is serde
// compatible under the exact keys this config used before (`cpu_high`,
// `mem_high`, `disk_high`), so the persisted JSON stays byte-compatible.
pub use xtop_plugin_api::AlertThresholds;

// ---------------------------------------------------------------------------
// Widget glyph style
// ---------------------------------------------------------------------------

/// Per-widget style overrides (key = widget name as used in layouts).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WidgetStyle {
    pub charset: Option<ChartCharset>,
    pub borders: Option<WidgetBorders>,
    /// Widget pack to render this name with (e.g. "default", "blocks").
    pub pack: Option<String>,
}

/// Global glyph style for widgets. Drives chart markers and block borders so
/// users can pick line/block/ascii rendering without touching code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UiStyle {
    pub charset: ChartCharset,
    pub borders: WidgetBorders,
    pub widgets: HashMap<String, WidgetStyle>,
    /// Widget pack used for every name without a per-widget override.
    pub pack: Option<String>,
}

impl UiStyle {
    /// Resolved charset for a widget (per-widget override beats global).
    pub fn charset_for(&self, widget: &str) -> ChartCharset {
        self.widgets
            .get(widget)
            .and_then(|w| w.charset)
            .unwrap_or(self.charset)
    }

    /// Resolved border style for a widget (per-widget override beats global).
    pub fn borders_for(&self, widget: &str) -> WidgetBorders {
        self.widgets
            .get(widget)
            .and_then(|w| w.borders)
            .unwrap_or(self.borders)
    }

    /// Resolved widget pack for a name (per-widget override beats global).
    pub fn pack_for(&self, widget: &str) -> Option<&str> {
        self.widgets
            .get(widget)
            .and_then(|w| w.pack.as_deref())
            .or(self.pack.as_deref())
    }
}

fn default_layout_mode() -> LayoutMode {
    LayoutMode::Dashboard
}

/// Default alert thresholds (90/90/90), mirroring the pre-contract values.
/// The contract type itself carries no `Default`: config-level defaults are a
/// kernel concern and stay here, next to the persisted schema.
pub fn default_alerts() -> AlertThresholds {
    AlertThresholds {
        cpu_high: 90.0,
        mem_high: 90.0,
        disk_high: 90.0,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    #[serde(default = "default_layout_mode")]
    pub layout_mode: LayoutMode,
    /// Layout name for custom layouts beyond the 7 built-in LayoutMode variants.
    /// If non-empty, takes precedence over `layout_mode`.
    #[serde(default)]
    pub layout_name: String,
    pub update_interval_ms: u64,
    pub history_points: usize,
    #[serde(default = "default_alerts")]
    pub alerts: AlertThresholds,
    #[serde(default)]
    pub keybindings: Keybindings,
    /// Widget glyph style (chart charset + borders). Optional; defaults to
    /// the classic look.
    #[serde(default)]
    pub style: UiStyle,
    /// Frame effect id (e.g. "fade"), consumed by the optional `effects`
    /// feature. Absent or unknown values disable effects; builds without the
    /// feature ignore this key entirely.
    #[serde(default)]
    pub effect: Option<String>,
    /// Plugin ids disabled by the user from the command bar.
    #[serde(default)]
    pub disabled_plugins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "x".to_string(),
            layout_mode: LayoutMode::Dashboard,
            layout_name: String::new(),
            update_interval_ms: 1000,
            history_points: 100,
            alerts: default_alerts(),
            keybindings: Keybindings::default(),
            style: UiStyle::default(),
            effect: None,
            disabled_plugins: Vec::new(),
        }
    }
}
