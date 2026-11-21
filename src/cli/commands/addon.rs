//! `xtop addon`: unified management of installable themes and layouts.
//!
//! These are external assets, not compiled kernel features. Plugin and widget
//! pack installation remains source-management tooling under `xtop plugin`
//! and `xtop widget`.

use crate::cli::help;
use crate::core::assets::{self, installed_asset, installed_by_name, AssetKind};
use crate::core::bootstrap::load_layout_defs;
use crate::core::theme::load_all_themes;

use super::download;

pub(crate) fn addon_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::ADDON);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::ADDON);
            Ok(())
        }
        "list" => {
            if help::wants_help(&args[3..]) {
                help::show(help::ADDON_LIST);
                return Ok(());
            }
            cmd_list();
            Ok(())
        }
        "available" => {
            if help::wants_help(&args[3..]) {
                help::show(help::ADDON_AVAILABLE);
                return Ok(());
            }
            cmd_available()?;
            Ok(())
        }
        "install" | "download" => {
            let help_text = if sub == "install" {
                help::ADDON_INSTALL
            } else {
                help::ADDON_DOWNLOAD
            };
            if help::wants_help(&args[3..]) {
                help::show(help_text);
                return Ok(());
            }
            let request = download::parse(&args[3..], help_text);
            let catalog = assets::fetch_catalog()?;
            let reports = assets::download_config(
                &catalog,
                request.all,
                request.query.as_deref(),
                request.force,
            )?;
            download::print_reports(&reports)
        }
        "path" => {
            if help::wants_help(&args[3..]) {
                help::show(help::ADDON_PATH);
                return Ok(());
            }
            println!("themes: {}", assets::themes_dir().display());
            println!("layouts: {}", assets::layouts_dir().display());
            Ok(())
        }
        other => help::usage_error(
            help::ADDON,
            &format!("xtop addon: unknown command '{other}'"),
        ),
    }
}

fn cmd_list() {
    println!("themes:");
    for theme in load_all_themes() {
        let source = if installed_asset(AssetKind::Theme, &theme.name).is_some() {
            "downloaded"
        } else {
            "local"
        };
        println!("  {} ({source})", theme.name);
    }
    println!("layouts:");
    for layout in load_layout_defs() {
        let source = if installed_by_name(AssetKind::Layout, &layout.name).is_some() {
            "downloaded"
        } else {
            "local"
        };
        println!("  {} ({source})", layout.name);
    }
}

fn cmd_available() -> anyhow::Result<()> {
    let catalog = assets::fetch_catalog()?;
    println!(
        "Catalog {} ({})",
        catalog.catalog_version, catalog.generated
    );
    println!("themes:");
    for asset in &catalog.themes {
        let state = if installed_asset(AssetKind::Theme, &asset.id).is_some() || asset.base {
            "installed"
        } else {
            "available"
        };
        println!(
            "  {} ({}) v{} {} {} [{}]",
            asset.name, asset.id, asset.version, asset.author, asset.license, state
        );
    }
    println!("layouts:");
    for asset in &catalog.layouts {
        let state = if installed_by_name(AssetKind::Layout, &asset.name).is_some() || asset.base {
            "installed"
        } else {
            "available"
        };
        println!(
            "  {} ({}) v{} {} {} [{}]",
            asset.name, asset.id, asset.version, asset.author, asset.license, state
        );
    }
    Ok(())
}
