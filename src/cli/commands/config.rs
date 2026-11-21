//! Config and asset management: catalog downloads, paths and base generation.

use crate::cli::help;
use crate::core::assets;
use crate::core::config::{self, Config};
use serde_json::{json, Value};

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
        "get" => cmd_get(args),
        "set" => cmd_set(args),
        "list" => cmd_list(args),
        "reset" => cmd_reset(args),
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

fn cmd_get(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[3..]) {
        help::show(help::CONFIG_GET);
        return Ok(());
    }
    let Some(path) = args.get(3) else {
        help::usage_error(help::CONFIG_GET, "Usage: xtop config get <key.path>");
    };
    let config = serde_json::to_value(config::load_config())
        .map_err(|error| anyhow::anyhow!("failed to encode config: {error}"))?;
    let Some(value) = get_path(&config, path) else {
        anyhow::bail!("config key '{path}' does not exist");
    };
    if let Some(text) = value.as_str() {
        println!("{text}");
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(value)
                .map_err(|error| anyhow::anyhow!("failed to encode value: {error}"))?
        );
    }
    Ok(())
}

fn cmd_set(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[3..]) {
        help::show(help::CONFIG_SET);
        return Ok(());
    }
    let Some(path) = args.get(3) else {
        help::usage_error(
            help::CONFIG_SET,
            "Usage: xtop config set <key.path> <value>",
        );
    };
    let Some(raw) = args.get(4) else {
        help::usage_error(
            help::CONFIG_SET,
            "Usage: xtop config set <key.path> <value>",
        );
    };
    if args.len() > 5 {
        help::usage_error(help::CONFIG_SET, "too many arguments");
    }
    let mut value = serde_json::to_value(config::load_config())
        .map_err(|error| anyhow::anyhow!("failed to encode config: {error}"))?;
    set_path(&mut value, path, parse_value(raw))?;
    let config: Config = serde_json::from_value(value)
        .map_err(|error| anyhow::anyhow!("invalid config value for '{path}': {error}"))?;
    if path == "theme" {
        validate_theme(&config.theme)?;
    }
    if path == "layout_name" && !config.layout_name.is_empty() {
        validate_layout(&config.layout_name)?;
    }
    config::save_config(&config)
        .map_err(|error| anyhow::anyhow!("failed to save config: {error}"))?;
    println!("Set {path}.");
    Ok(())
}

fn cmd_list(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[3..]) {
        help::show(help::CONFIG_LIST);
        return Ok(());
    }
    let config = serde_json::to_value(config::load_config())
        .map_err(|error| anyhow::anyhow!("failed to encode config: {error}"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&config)
            .map_err(|error| anyhow::anyhow!("failed to encode config: {error}"))?
    );
    Ok(())
}

fn cmd_reset(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[3..]) {
        help::show(help::CONFIG_RESET);
        return Ok(());
    }
    if !args[3..].iter().any(|arg| arg == "--yes") {
        help::usage_error(help::CONFIG_RESET, "pass --yes to reset config.json");
    }
    let mut config = Config::default();
    if let Some(layout) = crate::core::bootstrap::load_layout_defs().first() {
        config.layout_name = layout.name.clone();
    }
    config::save_config(&config)
        .map_err(|error| anyhow::anyhow!("failed to save config: {error}"))?;
    println!("Reset {} to defaults.", config::config_path().display());
    Ok(())
}

fn parse_value(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

fn get_path<'a>(mut value: &'a Value, path: &str) -> Option<&'a Value> {
    for part in path.split('.') {
        if part.is_empty() {
            return None;
        }
        value = match value {
            Value::Object(map) => map.get(part)?,
            Value::Array(items) => items.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(value)
}

fn set_path(root: &mut Value, path: &str, value: Value) -> anyhow::Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.iter().any(|part| part.is_empty()) {
        anyhow::bail!("invalid config path '{path}'");
    }
    let Some((last, parents)) = parts.split_last() else {
        anyhow::bail!("invalid config path '{path}'");
    };
    let mut current = root;
    for part in parents {
        if !current.is_object() {
            anyhow::bail!("config path '{path}' crosses a non-object value");
        }
        current = current
            .as_object_mut()
            .expect("checked object")
            .entry((*part).to_string())
            .or_insert_with(|| json!({}));
    }
    if !current.is_object() {
        anyhow::bail!("config path '{path}' crosses a non-object value");
    }
    current
        .as_object_mut()
        .expect("checked object")
        .insert((*last).to_string(), value);
    Ok(())
}

fn validate_theme(name: &str) -> anyhow::Result<()> {
    if crate::core::theme::load_all_themes()
        .iter()
        .any(|theme| theme.name == name)
    {
        return Ok(());
    }
    anyhow::bail!("unknown theme '{name}'; install it with 'xtop theme download {name}'")
}

fn validate_layout(name: &str) -> anyhow::Result<()> {
    if crate::core::bootstrap::load_layout_defs()
        .iter()
        .any(|layout| layout.name == name)
    {
        return Ok(());
    }
    anyhow::bail!("unknown layout '{name}'; install it with 'xtop layout download \"{name}\"'")
}
