//! Official asset catalog schema and lookup.
//!
//! `catalog.json` lives at the root of the kernel repository and lists the
//! downloadable themes and layouts with their version, author, license, file
//! name, raw URL and SHA-256. The catalog is data only; the binary keeps the
//! base theme and primary layout embedded.

use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

/// Kind of catalog asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Theme,
    Layout,
}

impl AssetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Theme => "theme",
            Self::Layout => "layout",
        }
    }

    pub fn dir(self) -> std::path::PathBuf {
        match self {
            Self::Theme => super::themes_dir(),
            Self::Layout => super::layouts_dir(),
        }
    }
}

/// Parsed `catalog.json`.
#[derive(Clone, Debug, Deserialize)]
pub struct Catalog {
    pub schema: u32,
    pub catalog_version: String,
    pub generated: String,
    #[serde(default)]
    pub base: CatalogBase,
    #[serde(default)]
    pub themes: Vec<Asset>,
    #[serde(default)]
    pub layouts: Vec<Asset>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CatalogBase {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub layout: String,
}

/// One downloadable asset entry.
#[derive(Clone, Debug, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub version: String,
    pub author: String,
    pub license: String,
    pub file: String,
    pub url: String,
    #[serde(default)]
    pub sha256: String,
    #[serde(default)]
    pub base: bool,
}

impl Catalog {
    pub fn parse(bytes: &[u8]) -> anyhow::Result<Self> {
        let catalog: Self = serde_json::from_slice(bytes)
            .map_err(|error| anyhow::anyhow!("invalid catalog JSON: {error}"))?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.schema != 1 {
            anyhow::bail!(
                "unsupported catalog schema {} (this build supports schema 1)",
                self.schema
            );
        }
        if self.catalog_version.trim().is_empty() {
            anyhow::bail!("catalog_version must not be empty");
        }
        if self.generated.trim().is_empty() {
            anyhow::bail!("catalog generated date must not be empty");
        }
        let mut ids = HashSet::new();
        for (kind, assets) in [
            (AssetKind::Theme, &self.themes),
            (AssetKind::Layout, &self.layouts),
        ] {
            for asset in assets {
                validate_asset(kind, asset)?;
                let key = format!("{}:{}", kind.as_str(), asset.id.to_ascii_lowercase());
                if !ids.insert(key) {
                    anyhow::bail!("duplicate {} id '{}' in catalog", kind.as_str(), asset.id);
                }
            }
        }
        if !self.base.theme.is_empty() && self.find_theme(&self.base.theme).is_none() {
            anyhow::bail!("catalog base theme '{}' is missing", self.base.theme);
        }
        if !self.base.layout.is_empty() && self.find_layout(&self.base.layout).is_none() {
            anyhow::bail!("catalog base layout '{}' is missing", self.base.layout);
        }
        Ok(())
    }

    pub fn find_matches(&self, query: &str) -> Vec<(AssetKind, &Asset)> {
        let needle = query.trim().to_ascii_lowercase();
        let mut matches = Vec::new();
        for asset in &self.themes {
            if asset.id.eq_ignore_ascii_case(&needle) || asset.name.eq_ignore_ascii_case(&needle) {
                matches.push((AssetKind::Theme, asset));
            }
        }
        for asset in &self.layouts {
            if asset.id.eq_ignore_ascii_case(&needle) || asset.name.eq_ignore_ascii_case(&needle) {
                matches.push((AssetKind::Layout, asset));
            }
        }
        matches
    }

    pub fn find_theme(&self, query: &str) -> Option<&Asset> {
        self.themes.iter().find(|asset| {
            asset.id.eq_ignore_ascii_case(query) || asset.name.eq_ignore_ascii_case(query)
        })
    }

    pub fn find_layout(&self, query: &str) -> Option<&Asset> {
        self.layouts.iter().find(|asset| {
            asset.id.eq_ignore_ascii_case(query) || asset.name.eq_ignore_ascii_case(query)
        })
    }

    pub fn assets(&self, kind: AssetKind) -> &[Asset] {
        match kind {
            AssetKind::Theme => &self.themes,
            AssetKind::Layout => &self.layouts,
        }
    }
}

fn validate_asset(kind: AssetKind, asset: &Asset) -> anyhow::Result<()> {
    if asset.id.trim().is_empty() {
        anyhow::bail!("{} entry has an empty id", kind.as_str());
    }
    if asset.name.trim().is_empty() {
        anyhow::bail!("{} '{}' has an empty name", kind.as_str(), asset.id);
    }
    for (label, value) in [
        ("version", &asset.version),
        ("author", &asset.author),
        ("license", &asset.license),
        ("url", &asset.url),
    ] {
        if value.trim().is_empty() {
            anyhow::bail!("{} '{}' has an empty {label}", kind.as_str(), asset.id);
        }
    }
    validate_file_name(kind, asset)?;
    if !asset.sha256.is_empty()
        && (asset.sha256.len() != 64 || !asset.sha256.chars().all(|c| c.is_ascii_hexdigit()))
    {
        anyhow::bail!("{} '{}' has an invalid sha256", kind.as_str(), asset.id);
    }
    Ok(())
}

fn validate_file_name(kind: AssetKind, asset: &Asset) -> anyhow::Result<()> {
    let path = Path::new(&asset.file);
    if path.file_name().and_then(|name| name.to_str()) != Some(asset.file.as_str()) {
        anyhow::bail!(
            "{} '{}' has an unsafe file name '{}'",
            kind.as_str(),
            asset.id,
            asset.file
        );
    }
    let extension = path.extension().and_then(|ext| ext.to_str());
    if !matches!(extension, Some("json") | Some("jsonc")) {
        anyhow::bail!(
            "{} '{}' must point to a .json or .jsonc file",
            kind.as_str(),
            asset.id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> String {
        r#"{
          "schema": 1,
          "catalog_version": "1.0.0",
          "generated": "2026-11-20",
          "themes": [{
            "id": "x", "name": "x", "version": "1.0.0", "author": "Xscriptor",
            "license": "GPL-3.0-or-later", "file": "x.jsonc",
            "url": "https://example.invalid/x.jsonc", "sha256": ""
          }],
          "layouts": [{
            "id": "dashboard", "name": "Dashboard", "category": "mode",
            "version": "1.0.0", "author": "Xscriptor",
            "license": "GPL-3.0-or-later", "file": "dashboard.jsonc",
            "url": "https://example.invalid/dashboard.jsonc", "sha256": ""
          }]
        }"#
        .to_string()
    }

    #[test]
    fn parses_and_finds_by_id_or_name() {
        let catalog = Catalog::parse(minimal().as_bytes()).expect("catalog");
        assert_eq!(catalog.find_theme("X").unwrap().id, "x");
        assert_eq!(catalog.find_layout("dashboard").unwrap().name, "Dashboard");
        assert!(catalog.find_theme("missing").is_none());
    }

    #[test]
    fn rejects_path_traversal_in_file_names() {
        let json = minimal().replace("\"file\": \"x.jsonc\"", "\"file\": \"../x.jsonc\"");
        let error = Catalog::parse(json.as_bytes()).unwrap_err().to_string();
        assert!(error.contains("unsafe file name"));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let json = minimal().replace(
            "\"themes\": [{",
            "\"themes\": [{\n            \"id\": \"x\", \"name\": \"dup\", \"version\": \"1.0.0\", \"author\": \"A\", \"license\": \"L\", \"file\": \"dup.jsonc\", \"url\": \"https://example.invalid/dup\", \"sha256\": \"\"\n          }, {",
        );
        let error = Catalog::parse(json.as_bytes()).unwrap_err().to_string();
        assert!(error.contains("duplicate theme id"));
    }

    #[test]
    fn rejects_unknown_schema() {
        let json = minimal().replace("\"schema\": 1", "\"schema\": 2");
        let error = Catalog::parse(json.as_bytes()).unwrap_err().to_string();
        assert!(error.contains("unsupported catalog schema"));
    }
    #[test]
    fn shipped_catalog_parses_and_theme_hashes_match() {
        use sha2::{Digest, Sha256};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(root.join("catalog.json")).expect("catalog.json");
        let catalog = Catalog::parse(&bytes).expect("shipped catalog must be valid");
        assert_eq!(catalog.base.theme, "x");
        assert_eq!(catalog.base.layout, "dashboard");
        assert!(catalog.themes.len() >= 12);
        assert!(catalog.layouts.len() >= 15);
        for theme in &catalog.themes {
            let path = root.join("assets/themes").join(&theme.file);
            let source =
                std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            let actual = format!("{:x}", Sha256::digest(&source));
            assert_eq!(actual, theme.sha256, "{} hash mismatch", theme.id);
        }
    }
}
