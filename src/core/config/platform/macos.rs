//! macOS config dir: `$HOME/Library/Application Support/xtop`.

use std::path::PathBuf;

use super::shared::env_path;

pub fn config_dir() -> PathBuf {
    let home = env_path(&["HOME"]).unwrap_or_else(|| PathBuf::from("."));
    home.join("Library")
        .join("Application Support")
        .join("xtop")
}
