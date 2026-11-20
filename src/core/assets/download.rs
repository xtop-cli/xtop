//! Fetch the official catalog and install themes or layouts.
//!
//! Network access is delegated to `curl` (present on Linux, macOS and modern
//! Windows); local paths and `file://` URLs are accepted for development and
//! tests. Downloads are verified against the catalog SHA-256 before the file
//! is moved into the user config directory.

use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::core::assets::catalog::{Asset, AssetKind, Catalog};
use crate::core::assets::installed;

pub const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/xtop-cli/xtop/main/catalog.json";

/// Where `xtop config download` looks for the catalog. Override with
/// `XTOP_CATALOG_URL` for local testing or a mirror.
pub fn catalog_url() -> String {
    std::env::var("XTOP_CATALOG_URL").unwrap_or_else(|_| DEFAULT_CATALOG_URL.to_string())
}

/// Result of one asset download attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Installed,
    Updated,
    Skipped,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct Report {
    pub kind: AssetKind,
    pub id: String,
    pub name: String,
    pub outcome: Outcome,
}

/// Fetch and validate the catalog.
pub fn fetch_catalog() -> anyhow::Result<Catalog> {
    let url = catalog_url();
    let bytes = fetch(&url)?;
    Catalog::parse(&bytes)
}

/// Download all assets of one kind.
pub fn download_kind(
    catalog: &Catalog,
    kind: AssetKind,
    all: bool,
    query: Option<&str>,
    force: bool,
) -> anyhow::Result<Vec<Report>> {
    if all {
        return Ok(download_assets(kind, catalog.assets(kind), force));
    }
    let Some(query) = query else {
        anyhow::bail!("provide an asset name or --all");
    };
    let matches: Vec<&Asset> = catalog
        .assets(kind)
        .iter()
        .filter(|asset| {
            asset.id.eq_ignore_ascii_case(query) || asset.name.eq_ignore_ascii_case(query)
        })
        .collect();
    match matches.as_slice() {
        [] => anyhow::bail!(
            "no {} named '{}' in the catalog; run 'xtop {} available'",
            kind.as_str(),
            query,
            kind.as_str()
        ),
        [asset] => Ok(vec![download_one(kind, asset, force)]),
        _ => anyhow::bail!(
            "ambiguous {} name '{}'; use the catalog id",
            kind.as_str(),
            query
        ),
    }
}

/// Download a theme or layout by name from the shared catalog.
pub fn download_config(
    catalog: &Catalog,
    all: bool,
    query: Option<&str>,
    force: bool,
) -> anyhow::Result<Vec<Report>> {
    if all {
        let mut reports = download_assets(AssetKind::Theme, &catalog.themes, force);
        reports.extend(download_assets(AssetKind::Layout, &catalog.layouts, force));
        return Ok(reports);
    }
    let Some(query) = query else {
        anyhow::bail!("provide an asset name or --all");
    };
    let matches = catalog.find_matches(query);
    if matches.is_empty() {
        anyhow::bail!("no theme or layout named '{query}' in the catalog");
    }
    Ok(matches
        .into_iter()
        .map(|(kind, asset)| download_one(kind, asset, force))
        .collect())
}

fn download_assets(kind: AssetKind, assets: &[Asset], force: bool) -> Vec<Report> {
    assets
        .iter()
        .map(|asset| download_one(kind, asset, force))
        .collect()
}

fn download_one(kind: AssetKind, asset: &Asset, force: bool) -> Report {
    let outcome = match install(kind, asset, force) {
        Ok(outcome) => outcome,
        Err(error) => Outcome::Failed(error.to_string()),
    };
    Report {
        kind,
        id: asset.id.clone(),
        name: asset.name.clone(),
        outcome,
    }
}

fn install(kind: AssetKind, asset: &Asset, force: bool) -> anyhow::Result<Outcome> {
    let dir = kind.dir();
    let dest = dir.join(&asset.file);
    let existed = dest.exists();
    if existed && !force {
        return Ok(Outcome::Skipped);
    }

    let bytes = fetch(&asset.url)?;
    verify_sha256(asset, &bytes)?;
    validate_content(kind, &bytes)?;
    write_atomic(&dest, &bytes)?;
    installed::record(kind, asset)?;
    Ok(if existed {
        Outcome::Updated
    } else {
        Outcome::Installed
    })
}

fn verify_sha256(asset: &Asset, bytes: &[u8]) -> anyhow::Result<()> {
    if asset.sha256.is_empty() {
        return Ok(());
    }
    let actual = format!("{:x}", Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(&asset.sha256) {
        anyhow::bail!(
            "asset '{}' checksum mismatch: expected {}, got {}",
            asset.id,
            asset.sha256,
            actual
        );
    }
    Ok(())
}

fn validate_content(kind: AssetKind, bytes: &[u8]) -> anyhow::Result<()> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| anyhow::anyhow!("asset is not valid UTF-8: {error}"))?;
    match kind {
        AssetKind::Theme => {
            crate::core::theme::parse_theme_source(source)
                .ok_or_else(|| anyhow::anyhow!("downloaded theme is not a valid JSONC theme"))?;
        }
        AssetKind::Layout => {
            xtop_layout::parse_layout_err(source)
                .map_err(|error| anyhow::anyhow!("downloaded layout is invalid: {error}"))?;
        }
    }
    Ok(())
}

fn fetch(url: &str) -> anyhow::Result<Vec<u8>> {
    if let Some(path) = url.strip_prefix("file://") {
        return fs::read(path).map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path));
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return curl(url);
    }
    if Path::new(url).is_file() {
        return fs::read(url).map_err(|error| anyhow::anyhow!("failed to read {url}: {error}"));
    }
    anyhow::bail!("unsupported URL '{url}'")
}

fn curl(url: &str) -> anyhow::Result<Vec<u8>> {
    let output = Command::new("curl")
        .args(["-fsSL", "--max-time", "60", url])
        .output()
        .map_err(|error| {
            anyhow::anyhow!("failed to run curl ({error}); install curl to download assets")
        })?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("curl failed for {url}: {}", message.trim());
    }
    Ok(output.stdout)
}

fn write_atomic(dest: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid destination {}", dest.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", parent.display()))?;
    let file_name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid destination {}", dest.display()))?;
    let tmp = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
    fs::write(&tmp, bytes)
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", tmp.display()))?;
    if dest.exists() {
        fs::remove_file(dest)
            .map_err(|error| anyhow::anyhow!("failed to replace {}: {error}", dest.display()))?;
    }
    fs::rename(&tmp, dest)
        .map_err(|error| anyhow::anyhow!("failed to move {}: {error}", dest.display()))?;
    Ok(())
}
