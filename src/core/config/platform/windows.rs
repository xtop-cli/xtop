//! Windows config dir: `%APPDATA%\xtop`.

use std::path::PathBuf;

use super::shared::env_path;

pub fn config_dir() -> PathBuf {
    env_path(&["APPDATA"]).unwrap_or_default().join("xtop")
}
