//! Theme management: list, show the current theme, set it, inspect the
//! catalog, download themes and print the themes directory.

use crate::cli::help;
use crate::core::assets::{
    self, installed_asset, AssetKind, BASE_AUTHOR, BASE_LICENSE, BASE_THEME_NAME, BASE_VERSION,
};
use crate::core::config;
use crate::core::theme::load_all_themes;

use super::download;

/// Handle `xtop theme <sub>`.
pub fn theme_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::THEME);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::THEME);
            Ok(())
        }
        "list" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_LIST);
                return Ok(());
            }
            cmd_list();
            Ok(())
        }
        "available" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_AVAILABLE);
                return Ok(());
            }
            cmd_available()?;
            Ok(())
        }
        "current" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_CURRENT);
                return Ok(());
            }
            println!("{}", config::load_config().theme);
            Ok(())
        }
        "set" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_SET);
                return Ok(());
            }
            let Some(name) = args.get(3) else {
                help::usage_error(help::THEME_SET, "Usage: xtop theme set <theme>");
            };
            set_theme(name)
        }
        "download" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_DOWNLOAD);
                return Ok(());
            }
            let request = download::parse(&args[3..], help::THEME_DOWNLOAD);
            let catalog = assets::fetch_catalog()?;
            let reports = assets::download_kind(
                &catalog,
                AssetKind::Theme,
                request.all,
                request.query.as_deref(),
                request.force,
            )?;
            download::print_reports(&reports)
        }
        "path" => {
            if help::wants_help(&args[3..]) {
                help::show(help::THEME_PATH);
                return Ok(());
            }
            println!("{}", assets::themes_dir().display());
            Ok(())
        }
        other => help::usage_error(
            help::THEME,
            &format!("xtop theme: unknown command '{other}'"),
        ),
    }
}

/// Resolve and persist a theme. Shared with `--change-theme`.
pub fn set_theme(name: &str) -> anyhow::Result<()> {
    let canonical = resolve_theme(name.trim())?;
    let mut cfg = config::load_config();
    cfg.theme = canonical.clone();
    config::save_config(&cfg).map_err(|e| anyhow::anyhow!("failed to save config: {e}"))?;
    println!("Theme set to '{canonical}'.");
    println!("Running xtop instances pick it up on the next tick.");
    Ok(())
}

fn resolve_theme(name: &str) -> anyhow::Result<String> {
    let themes = load_all_themes();
    if let Some(theme) = themes.iter().find(|t| t.name == name) {
        return Ok(theme.name.clone());
    }
    let mut available: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    available.sort_unstable();
    anyhow::bail!(
        "unknown theme '{name}'\navailable themes: {}\nRun 'xtop theme download {name}' to install it from the catalog.",
        available.join(", ")
    )
}

fn cmd_list() {
    println!(
        "{:<12} {:<12} {:<8} {:<12} LICENSE",
        "THEME", "SOURCE", "VERSION", "AUTHOR"
    );
    for theme in load_all_themes() {
        let installed = installed_asset(AssetKind::Theme, &theme.name);
        let source = if theme.name == BASE_THEME_NAME {
            "base"
        } else if installed.is_some() {
            "downloaded"
        } else {
            "user"
        };
        let version = installed.as_ref().map(|a| a.version.as_str()).unwrap_or(
            if theme.name == BASE_THEME_NAME {
                BASE_VERSION
            } else {
                "-"
            },
        );
        let author = installed.as_ref().map(|a| a.author.as_str()).unwrap_or(
            if theme.name == BASE_THEME_NAME {
                BASE_AUTHOR
            } else {
                "-"
            },
        );
        let license = installed.as_ref().map(|a| a.license.as_str()).unwrap_or(
            if theme.name == BASE_THEME_NAME {
                BASE_LICENSE
            } else {
                "-"
            },
        );
        println!(
            "{:<12} {:<12} {:<8} {:<12} {}",
            theme.name, source, version, author, license
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
        "{:<12} {:<12} {:<8} {:<12} {:<20} INSTALLED",
        "THEME", "NAME", "VERSION", "AUTHOR", "LICENSE"
    );
    for asset in &catalog.themes {
        let installed = installed_asset(AssetKind::Theme, &asset.id).is_some();
        let state = if asset.id == BASE_THEME_NAME {
            "base"
        } else if installed {
            "yes"
        } else {
            "no"
        };
        println!(
            "{:<12} {:<12} {:<8} {:<12} {:<20} {}",
            asset.id, asset.name, asset.version, asset.author, asset.license, state
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_theme_resolves() {
        assert_eq!(resolve_theme("x").unwrap(), "x");
    }

    #[test]
    fn unknown_theme_lists_available_names_and_download_hint() {
        let err = resolve_theme("no-such-theme").unwrap_err().to_string();
        assert!(err.contains("unknown theme 'no-such-theme'"));
        assert!(
            err.contains("x"),
            "available list must include the built-in"
        );
        assert!(err.contains("xtop theme download no-such-theme"));
    }
}
