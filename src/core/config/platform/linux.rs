//! Linux config dir: `$XDG_CONFIG_HOME/xtop`, falling back to
//! `$HOME/.config/xtop`.

use std::path::PathBuf;

use super::shared::env_path;

pub fn config_dir() -> PathBuf {
    env_path(&["XDG_CONFIG_HOME"])
        .unwrap_or_else(|| {
            env_path(&["HOME"])
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("xtop")
}
