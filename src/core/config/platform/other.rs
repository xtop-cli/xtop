//! Fallback config dir for other Unix-likes: `$HOME/.config/xtop`.

use std::path::PathBuf;

use super::shared::env_path;

pub fn config_dir() -> PathBuf {
    env_path(&["HOME"])
        .unwrap_or_default()
        .join(".config")
        .join("xtop")
}
