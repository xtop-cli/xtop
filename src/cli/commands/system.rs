//! `xtop info` and `xtop doctor`: diagnostics without starting the TUI.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cli::help;
use crate::core::assets;
use crate::core::config::{self, Config};
use serde_json::{json, Value};

pub(crate) fn info_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::INFO);
        return Ok(());
    }
    let mut json_output = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "--json" => json_output = true,
            other => help::usage_error(help::INFO, &format!("unknown option '{other}'")),
        }
    }
    let value = info_value();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
        );
    } else {
        println!("xtop {}", env!("CARGO_PKG_VERSION"));
        println!(
            "target: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        println!("features: {}", enabled_features().join(", "));
        println!("config:    {}", config::config_dir().display());
        println!("themes:    {}", assets::themes_dir().display());
        println!("layouts:   {}", assets::layouts_dir().display());
        println!("installed: {}", assets::installed_manifest_path().display());
        println!("catalog:   {}", assets::catalog_url());
        println!(
            "asset files: {} themes, {} layouts",
            count_assets(&assets::themes_dir()),
            count_assets(&assets::layouts_dir())
        );
        println!(
            "tools: curl={} git={} cargo={}",
            yes_no(tool_available("curl")),
            yes_no(tool_available("git")),
            yes_no(tool_available("cargo"))
        );
    }
    Ok(())
}

pub(crate) fn doctor_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::DOCTOR);
        return Ok(());
    }
    let mut json_output = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "--json" => json_output = true,
            other => help::usage_error(help::DOCTOR, &format!("unknown option '{other}'")),
        }
    }

    let checks: Vec<Check> = vec![
        check_dir_writable("config directory", &config::config_dir()),
        check_config_parse(),
        check_base_asset(
            "base theme (themes/x.jsonc)",
            &assets::themes_dir().join("x.jsonc"),
        ),
        check_base_asset(
            "base layout (layouts/dashboard.jsonc)",
            &assets::layouts_dir().join("dashboard.jsonc"),
        ),
        check_manifest(),
        check_tool("curl (asset downloads)", "curl"),
        check_tool("git (developer commands)", "git"),
        check_tool("cargo (developer commands)", "cargo"),
        Check::ok("catalog URL", assets::catalog_url()),
    ];

    let failed = checks.iter().any(|check| check.status == Status::Fail);
    if json_output {
        let value = json!({
            "schema": 1,
            "ok": !failed,
            "checks": checks.iter().map(Check::value).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&value)
                .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
        );
    } else {
        for check in &checks {
            println!(
                "[{}] {}: {}",
                check.status.label(),
                check.name,
                check.detail
            );
        }
    }
    if failed {
        anyhow::bail!("doctor found one or more failed checks");
    }
    Ok(())
}

fn info_value() -> Value {
    json!({
        "schema": 1,
        "version": env!("CARGO_PKG_VERSION"),
        "target": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        },
        "features": enabled_features(),
        "paths": {
            "config": config::config_dir(),
            "themes": assets::themes_dir(),
            "layouts": assets::layouts_dir(),
            "installed_manifest": assets::installed_manifest_path(),
            "catalog": assets::catalog_url(),
        },
        "asset_files": {
            "themes": count_assets(&assets::themes_dir()),
            "layouts": count_assets(&assets::layouts_dir()),
        },
        "tools": {
            "curl": tool_available("curl"),
            "git": tool_available("git"),
            "cargo": tool_available("cargo"),
        },
    })
}

fn enabled_features() -> Vec<&'static str> {
    let mut features = Vec::new();
    if cfg!(feature = "plugin-samurai") {
        features.push("plugin-samurai");
    }
    if cfg!(feature = "mcp-extension") {
        features.push("mcp-extension");
    }
    if cfg!(feature = "widget-blocks") {
        features.push("widget-blocks");
    }
    if cfg!(feature = "effects") {
        features.push("effects");
    }
    if cfg!(feature = "plugin-wasm") {
        features.push("plugin-wasm");
    }
    if cfg!(feature = "plugin-external") {
        features.push("plugin-external");
    }
    features
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    Warn,
    Fail,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

struct Check {
    name: &'static str,
    status: Status,
    detail: String,
}

impl Check {
    fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: Status::Ok,
            detail: detail.into(),
        }
    }

    fn warn(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: Status::Warn,
            detail: detail.into(),
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            name,
            status: Status::Fail,
            detail: detail.into(),
        }
    }

    fn value(&self) -> Value {
        json!({
            "name": self.name,
            "status": self.status.as_str(),
            "detail": self.detail,
        })
    }
}

fn check_dir_writable(name: &'static str, dir: &Path) -> Check {
    if let Err(error) = fs::create_dir_all(dir) {
        return Check::fail(name, format!("cannot create {}: {error}", dir.display()));
    }
    let probe = dir.join(format!(".xtop-doctor-{}", std::process::id()));
    let result = fs::write(&probe, b"ok").and_then(|_| fs::remove_file(&probe));
    match result {
        Ok(()) => Check::ok(name, dir.display().to_string()),
        Err(error) => Check::fail(name, format!("cannot write in {}: {error}", dir.display())),
    }
}

fn check_config_parse() -> Check {
    let path = config::config_path();
    if !path.exists() {
        return Check::warn(
            "config.json",
            format!("missing at {}; run 'xtop --gen-config'", path.display()),
        );
    }
    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(error) => return Check::fail("config.json", format!("cannot read: {error}")),
    };
    match serde_json::from_str::<Config>(&data) {
        Ok(_) => Check::ok("config.json", path.display().to_string()),
        Err(error) => Check::fail("config.json", format!("invalid JSON schema: {error}")),
    }
}

fn check_base_asset(name: &'static str, path: &Path) -> Check {
    if path.is_file() {
        Check::ok(name, path.display().to_string())
    } else {
        Check::warn(
            name,
            format!("missing at {}; run 'xtop --gen-config'", path.display()),
        )
    }
}

fn check_manifest() -> Check {
    let path = assets::installed_manifest_path();
    if !path.exists() {
        return Check::ok("assets.json", "not present; no catalog assets installed");
    }
    match fs::read_to_string(&path).and_then(|data| {
        serde_json::from_str::<serde_json::Value>(&data)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }) {
        Ok(_) => Check::ok("assets.json", path.display().to_string()),
        Err(error) => Check::fail("assets.json", format!("invalid manifest: {error}")),
    }
}

fn check_tool(name: &'static str, tool: &str) -> Check {
    if tool_available(tool) {
        Check::ok(name, "available")
    } else {
        Check::warn(name, format!("'{tool}' not found on PATH"))
    }
}

fn tool_available(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn count_assets(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| matches!(ext, "json" | "jsonc"))
                })
                .count()
        })
        .unwrap_or(0)
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
