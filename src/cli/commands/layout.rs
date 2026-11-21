//! Layout management: list installed layouts, set the active layout, inspect
//! the catalog, download layouts and validate layout files.

use std::fs;
use std::path::Path;

use crate::cli::help;
use crate::core::assets::{
    self, installed_by_name, AssetKind, BASE_AUTHOR, BASE_LICENSE, BASE_VERSION,
};
use crate::core::bootstrap::load_layout_defs;
use crate::core::config;

use super::download;

/// Handle `xtop layout <sub>`.
pub fn layout_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::LAYOUT);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::LAYOUT);
            Ok(())
        }
        "list" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_LIST);
                return Ok(());
            }
            cmd_list();
            Ok(())
        }
        "available" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_AVAILABLE);
                return Ok(());
            }
            cmd_available()?;
            Ok(())
        }
        "current" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_CURRENT);
                return Ok(());
            }
            cmd_current();
            Ok(())
        }
        "show" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_SHOW);
                return Ok(());
            }
            let Some(name) = args.get(3) else {
                help::usage_error(help::LAYOUT_SHOW, "Usage: xtop layout show <layout>");
            };
            cmd_show(name)
        }
        "set" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_SET);
                return Ok(());
            }
            let Some(name) = args.get(3) else {
                help::usage_error(help::LAYOUT_SET, "Usage: xtop layout set <layout>");
            };
            set_layout(name)
        }
        "download" | "install" => {
            let help_text = if sub == "install" {
                help::LAYOUT_INSTALL
            } else {
                help::LAYOUT_DOWNLOAD
            };
            if help::wants_help(&args[3..]) {
                help::show(help_text);
                return Ok(());
            }
            let request = download::parse(&args[3..], help_text);
            let catalog = assets::fetch_catalog()?;
            let reports = assets::download_kind(
                &catalog,
                AssetKind::Layout,
                request.all,
                request.query.as_deref(),
                request.force,
            )?;
            download::print_reports(&reports)
        }
        "check" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_CHECK);
                return Ok(());
            }
            let Some(path) = args.get(3) else {
                help::usage_error(
                    help::LAYOUT_CHECK,
                    "Usage: xtop layout check <file.jsonc|file.json>",
                );
            };
            cmd_check(Path::new(path))
        }
        "path" => {
            if help::wants_help(&args[3..]) {
                help::show(help::LAYOUT_PATH);
                return Ok(());
            }
            println!("{}", assets::layouts_dir().display());
            Ok(())
        }
        other => help::usage_error(
            help::LAYOUT,
            &format!("xtop layout: unknown command '{other}'"),
        ),
    }
}

fn set_layout(name: &str) -> anyhow::Result<()> {
    let defs = load_layout_defs();
    let needle = name.trim();
    let exact = defs.iter().find(|def| def.name == needle);
    let case_insensitive = || {
        defs.iter()
            .find(|def| def.name.eq_ignore_ascii_case(needle))
    };
    let Some(def) = exact.or_else(case_insensitive) else {
        let mut available: Vec<&str> = defs.iter().map(|def| def.name.as_str()).collect();
        available.sort_unstable();
        anyhow::bail!(
            "unknown layout '{name}'\navailable layouts: {}\nRun 'xtop layout download {name}' to install it from the catalog.",
            available.join(", ")
        );
    };

    let mut cfg = config::load_config();
    cfg.layout_name = def.name.clone();
    cfg.layout_mode = xtop_layout::layout_mode_for_name(&def.name, cfg.layout_mode);
    config::save_config(&cfg).map_err(|e| anyhow::anyhow!("failed to save config: {e}"))?;
    println!("Layout set to '{}'.", def.name);
    println!("Running xtop instances pick it up on the next tick.");
    Ok(())
}

fn cmd_current() {
    let cfg = config::load_config();
    if cfg.layout_name.is_empty() {
        println!("{}", cfg.layout_mode.label());
    } else {
        println!("{}", cfg.layout_name);
    }
}

fn cmd_list() {
    let current = {
        let cfg = config::load_config();
        if cfg.layout_name.is_empty() {
            cfg.layout_mode.label().to_string()
        } else {
            cfg.layout_name
        }
    };
    println!(
        "{:<2} {:<22} {:<12} {:<8} {:<12} LICENSE",
        "", "LAYOUT", "SOURCE", "VERSION", "AUTHOR"
    );
    for def in load_layout_defs() {
        let installed = installed_by_name(AssetKind::Layout, &def.name);
        let source = if def.name == "Dashboard" {
            "base"
        } else if installed.is_some() {
            "downloaded"
        } else {
            "user"
        };
        let marker = if def.name == current { "*" } else { "" };
        let base = def.name == "Dashboard";
        let version = installed
            .as_ref()
            .map(|a| a.version.as_str())
            .unwrap_or(if base { BASE_VERSION } else { "-" });
        let author = installed
            .as_ref()
            .map(|a| a.author.as_str())
            .unwrap_or(if base { BASE_AUTHOR } else { "-" });
        let license = installed
            .as_ref()
            .map(|a| a.license.as_str())
            .unwrap_or(if base { BASE_LICENSE } else { "-" });
        println!(
            "{:<2} {:<22} {:<12} {:<8} {:<12} {}",
            marker, def.name, source, version, author, license
        );
    }
}

fn cmd_available() -> anyhow::Result<()> {
    let catalog = assets::fetch_catalog()?;
    println!(
        "Catalog {} ({})",
        catalog.catalog_version, catalog.generated
    );
    println!(
        "{:<20} {:<22} {:<10} {:<8} {:<12} {:<20} INSTALLED",
        "ID", "NAME", "CATEGORY", "VERSION", "AUTHOR", "LICENSE"
    );
    for asset in &catalog.layouts {
        let installed = installed_by_name(AssetKind::Layout, &asset.name).is_some();
        let state = if asset.id == catalog.base.layout {
            "base"
        } else if installed {
            "yes"
        } else {
            "no"
        };
        println!(
            "{:<20} {:<22} {:<10} {:<8} {:<12} {:<20} {}",
            asset.id,
            asset.name,
            asset.category.as_deref().unwrap_or("-"),
            asset.version,
            asset.author,
            asset.license,
            state
        );
    }
    Ok(())
}

/// Validate a layout file against the schema.
fn cmd_check(path: &Path) -> anyhow::Result<()> {
    let data = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", path.display()))?;
    match xtop_layout::parse_layout_err(&data) {
        Ok(def) => {
            println!("OK  {} -> layout \"{}\" is valid", path.display(), def.name);
            Ok(())
        }
        Err(e) => anyhow::bail!("INVALID {} -> {e}", path.display()),
    }
}

fn cmd_show(name: &str) -> anyhow::Result<()> {
    let defs = load_layout_defs();
    let Some(def) = defs
        .iter()
        .find(|def| def.name == name)
        .or_else(|| defs.iter().find(|def| def.name.eq_ignore_ascii_case(name)))
    else {
        anyhow::bail!("unknown layout '{name}'; install it with 'xtop layout download \"{name}\"'");
    };
    let installed = installed_by_name(AssetKind::Layout, &def.name);
    let source = if def.name == "Dashboard" {
        "base"
    } else if installed.is_some() {
        "downloaded"
    } else {
        "user"
    };
    println!("name: {}", def.name);
    println!("source: {source}");
    if let Some((path, _)) = layout_source(&def.name) {
        println!("path: {}", path.display());
    }
    if let Some(installed) = installed {
        println!(
            "version: {}  author: {}  license: {}",
            installed.version, installed.author, installed.license
        );
    }
    println!();
    match layout_source(&def.name) {
        Some((_, source)) => print!("{source}"),
        None => println!("(source file not found; using the embedded definition)"),
    }
    Ok(())
}

fn layout_source(name: &str) -> Option<(std::path::PathBuf, String)> {
    let dir = assets::layouts_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let extension = path.extension().and_then(|ext| ext.to_str());
            if !matches!(extension, Some("json") | Some("jsonc")) {
                continue;
            }
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            if xtop_layout::parse_layout(&source)
                .map(|layout| layout.name == name)
                .unwrap_or(false)
            {
                return Some((path, source));
            }
        }
    }
    if name == "Dashboard" {
        for &(stem, source) in assets::base_layout_sources() {
            if stem == "dashboard" {
                return Some((
                    std::path::PathBuf::from("<embedded dashboard>"),
                    source.to_string(),
                ));
            }
        }
    }
    None
}
