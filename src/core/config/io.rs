//! Config file read/write against the persisted schema.

use std::fs;
use std::path::PathBuf;

use super::schema::Config;
use crate::core::config::config_dir;

/// Path of the persisted config file.
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load the config, falling back to defaults when missing or invalid.
pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(mut cfg) = serde_json::from_str::<Config>(&data) {
            // Guard against a busy-looping event loop (interval 0 = spin at
            // 100% CPU sampling the system every frame).
            cfg.update_interval_ms = cfg.update_interval_ms.clamp(100, 3_600_000);
            // The command bar replaces the old `/` process-search key. Keep
            // existing configs usable by moving that binding once in memory.
            if let Some(position) = cfg.keybindings.search.iter().position(|key| key == "/") {
                cfg.keybindings.search.remove(position);
                if cfg.keybindings.search.is_empty() {
                    cfg.keybindings.search.push("ctrl+f".into());
                }
                if !cfg.keybindings.command_palette.iter().any(|key| key == "/") {
                    cfg.keybindings.command_palette.push("/".into());
                }
            }
            return cfg;
        }
    }
    Config::default()
}

/// Save the config to disk.
pub fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}
