//! Local manifest of downloaded assets.
//!
//! The catalog is the remote source of truth; this file records what has been
//! installed locally so `theme list` and `layout list` can show version,
//! author and license without network access.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::config;

use super::catalog::{Asset, AssetKind};

pub const MANIFEST_SCHEMA: u32 = 1;

/// One installed asset entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledAsset {
    pub name: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub file: String,
    #[serde(default)]
    pub sha256: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstalledManifest {
    #[serde(default)]
    pub schema: u32,
    #[serde(default)]
    pub themes: BTreeMap<String, InstalledAsset>,
    #[serde(default)]
    pub layouts: BTreeMap<String, InstalledAsset>,
}

/// Path of the installed-assets manifest.
pub fn installed_manifest_path() -> PathBuf {
    config::config_dir().join("assets.json")
}

/// Load the manifest; a missing or invalid file yields an empty manifest.
pub fn load() -> InstalledManifest {
    let Ok(data) = fs::read_to_string(installed_manifest_path()) else {
        return InstalledManifest::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Record a downloaded asset in the manifest.
pub fn record(kind: AssetKind, asset: &Asset) -> anyhow::Result<()> {
    let mut manifest = load();
    manifest.schema = MANIFEST_SCHEMA;
    let installed = InstalledAsset {
        name: asset.name.clone(),
        version: asset.version.clone(),
        author: asset.author.clone(),
        license: asset.license.clone(),
        file: asset.file.clone(),
        sha256: asset.sha256.clone(),
    };
    match kind {
        AssetKind::Theme => {
            manifest.themes.insert(asset.id.clone(), installed);
        }
        AssetKind::Layout => {
            manifest.layouts.insert(asset.id.clone(), installed);
        }
    }
    save(&manifest)
}

/// Installed metadata for one asset, if recorded.
pub fn get(kind: AssetKind, id: &str) -> Option<InstalledAsset> {
    let manifest = load();
    match kind {
        AssetKind::Theme => manifest.themes.get(id).cloned(),
        AssetKind::Layout => manifest.layouts.get(id).cloned(),
    }
}

/// Find installed metadata by display name.
pub fn find_by_name(kind: AssetKind, name: &str) -> Option<InstalledAsset> {
    let manifest = load();
    let assets = match kind {
        AssetKind::Theme => &manifest.themes,
        AssetKind::Layout => &manifest.layouts,
    };
    assets
        .values()
        .find(|asset| asset.name.eq_ignore_ascii_case(name))
        .cloned()
}

fn save(manifest: &InstalledManifest) -> anyhow::Result<()> {
    let path = installed_manifest_path();
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid manifest path"))?;
    fs::create_dir_all(parent)
        .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", parent.display()))?;
    let data = serde_json::to_string_pretty(manifest)
        .map_err(|error| anyhow::anyhow!("failed to encode installed manifest: {error}"))?;
    let tmp = parent.join(format!(".assets.json.tmp-{}", std::process::id()));
    fs::write(&tmp, data)
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|error| anyhow::anyhow!("failed to replace {}: {error}", path.display()))?;
    }
    fs::rename(&tmp, &path)
        .map_err(|error| anyhow::anyhow!("failed to move {}: {error}", path.display()))?;
    Ok(())
}
