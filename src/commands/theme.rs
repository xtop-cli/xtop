//! `xtop --ct <theme>`: change the active theme from the command line.
//!
//! The command persists the choice into the user config (`config.json`).
//! When no instance is running, the next `xtop` start picks it up from the
//! config; a running instance follows the change live because the TUI run
//! loop polls the persisted theme at tick boundaries. That file-based
//! contract is the sync hook for external themers (e.g. a Hyprland theme
//! switcher script running `xtop --ct <name>`).

use crate::config;
use crate::theme::load_all_themes;

/// Resolve a theme name against the loaded themes (built-in + user files).
fn resolve_theme(name: &str) -> anyhow::Result<String> {
    let themes = load_all_themes();
    if let Some(theme) = themes.iter().find(|t| t.name == name) {
        return Ok(theme.name.clone());
    }
    let mut available: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    available.sort_unstable();
    anyhow::bail!(
        "unknown theme '{name}'\navailable themes: {}",
        available.join(", ")
    )
}

/// Persist `name` as the active theme.
pub fn cmd_change_theme(name: &str) -> anyhow::Result<()> {
    let name = name.trim();
    let canonical = resolve_theme(name)?;

    let mut cfg = config::load_config();
    cfg.theme = canonical.clone();
    config::save_config(&cfg).map_err(|e| anyhow::anyhow!("failed to save config: {e}"))?;

    println!("Theme set to '{canonical}'.");
    println!("Running xtop instances pick it up on the next tick.");
    Ok(())
}

/// Theme persisted in the user config, or `None` when the file is missing or
/// does not parse (a partial/invalid file must never reset the live theme).
pub fn persisted_theme() -> Option<String> {
    let data = std::fs::read_to_string(config::config_path()).ok()?;
    let cfg: config::Config = serde_json::from_str(&data).ok()?;
    Some(cfg.theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_theme_resolves() {
        assert_eq!(resolve_theme("x").unwrap(), "x");
    }

    #[test]
    fn unknown_theme_lists_available_names() {
        let err = resolve_theme("no-such-theme").unwrap_err().to_string();
        assert!(err.contains("unknown theme 'no-such-theme'"));
        assert!(
            err.contains("x"),
            "available list must include the built-in"
        );
    }
}
