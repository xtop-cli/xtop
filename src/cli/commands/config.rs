//! Config and asset management: catalog downloads, paths and base generation.

use crate::cli::help;
use crate::core::assets;
use crate::core::config;

use super::download;

/// Handle `xtop config <sub>`.
pub fn config_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::CONFIG);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::CONFIG);
            Ok(())
        }
        "download" => {
            if help::wants_help(&args[3..]) {
                help::show(help::CONFIG_DOWNLOAD);
                return Ok(());
            }
            let request = download::parse(&args[3..], help::CONFIG_DOWNLOAD);
            let catalog = assets::fetch_catalog()?;
            let reports = assets::download_config(
                &catalog,
                request.all,
                request.query.as_deref(),
                request.force,
            )?;
            download::print_reports(&reports)
        }
        "generate" => {
            if help::wants_help(&args[3..]) {
                help::show(help::CONFIG_GENERATE);
                return Ok(());
            }
            generate_base_config()
        }
        "path" => {
            if help::wants_help(&args[3..]) {
                help::show(help::CONFIG_PATH);
                return Ok(());
            }
            println!("config:    {}", config::config_dir().display());
            println!("themes:    {}", assets::themes_dir().display());
            println!("layouts:   {}", assets::layouts_dir().display());
            println!("installed: {}", assets::installed_manifest_path().display());
            Ok(())
        }
        other => help::usage_error(
            help::CONFIG,
            &format!("xtop config: unknown command '{other}'"),
        ),
    }
}

/// Create the base config, theme and layout. Shared with `--gen-config`.
pub fn generate_base_config() -> anyhow::Result<()> {
    for seeded in assets::generate_base_config()? {
        let state = if seeded.created { "Created" } else { "Exists " };
        println!("{state} {}", seeded.path.display());
    }
    Ok(())
}
