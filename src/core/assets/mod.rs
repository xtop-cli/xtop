//! External asset management.
//!
//! The binary embeds only the base `x` theme and the primary Dashboard layout.
//! Everything else is fetched on demand from the official catalog into the
//! user config directory:
//!
//! - `base`: embedded base assets and `--gen-config`
//! - `catalog`: `catalog.json` schema, validation and lookup
//! - `download`: curl/file fetch, SHA-256 verification and atomic install
//! - `installed`: local `assets.json` metadata for downloaded assets
//!
//! Layouts are read from `layouts_dir()` and themes from `themes_dir()`; both
//! are plain JSONC files and can be edited by the user.

mod base;
mod catalog;
mod download;
mod installed;

pub use base::{
    base_layout_sources, base_theme_source, ensure_default_assets, generate_base_config,
    layouts_dir, themes_dir, SeededPath, BASE_AUTHOR, BASE_LICENSE, BASE_THEME_NAME, BASE_VERSION,
};
pub use catalog::{Asset, AssetKind, Catalog, CatalogBase};
pub(crate) use download::fetch_url;
pub use download::{
    catalog_url, download_config, download_kind, fetch_catalog, Outcome, Report,
    DEFAULT_CATALOG_URL,
};
pub use installed::{
    find_by_name as installed_by_name, get as installed_asset, installed_manifest_path, record,
    InstalledAsset,
};
