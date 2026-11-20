//! Base assets shipped with the binary.
//!
//! Only the `x` theme and the primary `Dashboard` layout stay embedded. Every
//! other theme or layout is external and is installed on demand from the
//! official catalog into the user config directory.

use std::fs;
use std::path::{Path, PathBuf};

use crate::core::config::{self, Config};
use crate::core::theme;

/// Name and source of the embedded base theme.
pub const BASE_THEME_NAME: &str = "x";
/// Catalog metadata for the embedded base assets.
pub const BASE_VERSION: &str = "1.0.0";
pub const BASE_AUTHOR: &str = "Xscriptor";
pub const BASE_LICENSE: &str = "GPL-3.0-or-later";
const BASE_THEME_SOURCE: &str = include_str!("../../../assets/themes/x.jsonc");

/// One file produced by [`generate_base_config`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeededPath {
    pub path: PathBuf,
    pub created: bool,
}

/// Directory holding user themes.
pub fn themes_dir() -> PathBuf {
    theme::themes_dir()
}

/// Directory holding user layouts.
pub fn layouts_dir() -> PathBuf {
    config::config_dir().join("layouts")
}

/// Seed the embedded base assets without printing or failing.
///
/// Files are created only when missing; user edits are never overwritten.
/// This is the automatic first-run path used by the TUI and MCP commands.
pub fn ensure_default_assets() {
    let _ = seed_base_assets();
}

/// Create `config.json`, the base theme and the primary layout if missing.
///
/// This backs the explicit `--gen-config` flag. It reports every path so the
/// caller can distinguish newly created files from files already present.
pub fn generate_base_config() -> anyhow::Result<Vec<SeededPath>> {
    let mut seeded = seed_base_assets()?;

    let config_path = config::config_path();
    let created = if config_path.exists() {
        false
    } else {
        config::save_config(&Config::default()).map_err(|error| {
            anyhow::anyhow!("failed to write {}: {error}", config_path.display())
        })?;
        true
    };
    seeded.insert(
        0,
        SeededPath {
            path: config_path,
            created,
        },
    );
    Ok(seeded)
}

/// Base theme source embedded in the binary (used by tests).
pub fn base_theme_source() -> &'static str {
    BASE_THEME_SOURCE
}

/// Base layout sources embedded in `xtop-layout` (currently Dashboard only).
pub fn base_layout_sources() -> &'static [(&'static str, &'static str)] {
    xtop_layout::default_layout_sources
}

fn seed_base_assets() -> anyhow::Result<Vec<SeededPath>> {
    let mut seeded = Vec::new();
    let themes = themes_dir();
    seeded.push(seed_file(
        &themes.join(format!("{BASE_THEME_NAME}.jsonc")),
        BASE_THEME_SOURCE,
    )?);
    let layouts = layouts_dir();
    for &(name, source) in base_layout_sources() {
        seeded.push(seed_file(&layouts.join(format!("{name}.jsonc")), source)?);
    }
    Ok(seeded)
}

fn seed_file(path: &Path, content: &str) -> anyhow::Result<SeededPath> {
    if path.exists() {
        return Ok(SeededPath {
            path: path.to_path_buf(),
            created: false,
        });
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(path, content)
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", path.display()))?;
    Ok(SeededPath {
        path: path.to_path_buf(),
        created: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_theme_parses_and_is_x() {
        let theme = crate::core::theme::parse_theme_source(base_theme_source())
            .expect("embedded x theme must parse");
        assert_eq!(theme.name, BASE_THEME_NAME);
    }

    #[test]
    fn only_dashboard_is_embedded() {
        let layouts = base_layout_sources();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].0, "dashboard");
    }
}
