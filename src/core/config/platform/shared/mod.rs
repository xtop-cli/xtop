//! Helpers shared by all platform config-dir implementations.

use std::path::PathBuf;

/// First environment variable from `names` that resolves to a path.
pub(crate) fn env_path(names: &[&str]) -> Option<PathBuf> {
    names
        .iter()
        .find_map(|n| std::env::var_os(n).map(PathBuf::from))
}
