//! `xtop update`: update the running installation through the medium that
//! installed it (xpm, pacman, cargo or a standalone release binary).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::help;
use crate::core::assets;
use serde_json::{json, Value};

const REPO: &str = "https://github.com/xtop-cli/xtop";
const RELEASE_API: &str = "https://api.github.com/repos/xtop-cli/xtop/releases/latest";
const SHELL_INSTALLER: &str =
    "https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.sh";
const POWERSHELL_INSTALLER: &str =
    "https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.ps1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MethodChoice {
    Auto,
    Xpm,
    Pacman,
    Cargo,
    Binary,
}

struct Options {
    check: bool,
    dry_run: bool,
    yes: bool,
    force: bool,
    all_features: bool,
    json: bool,
    version: Option<String>,
    method: MethodChoice,
}

#[derive(Clone, Debug)]
enum Medium {
    Xpm { package: String },
    Pacman { package: String },
    Cargo,
    Binary,
}

impl Medium {
    fn name(&self) -> &'static str {
        match self {
            Self::Xpm { .. } => "xpm",
            Self::Pacman { .. } => "pacman",
            Self::Cargo => "cargo",
            Self::Binary => "binary",
        }
    }

    fn package(&self) -> Option<&str> {
        match self {
            Self::Xpm { package } | Self::Pacman { package } => Some(package),
            Self::Cargo | Self::Binary => None,
        }
    }
}

struct Plan {
    program: Option<String>,
    args: Vec<String>,
    display: String,
    binary_script: Option<&'static str>,
    needs_privilege: bool,
}

pub(crate) fn update_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::UPDATE);
        return Ok(());
    }
    let options = parse_options(&args[2..])?;
    let medium = resolve_medium(options.method)?;

    if options.all_features && !matches!(medium, Medium::Cargo) {
        anyhow::bail!("--all-features is only valid for a cargo installation");
    }
    if options.version.is_some() && matches!(medium, Medium::Xpm { .. } | Medium::Pacman { .. }) {
        anyhow::bail!("--version is only supported for cargo and standalone binary installs");
    }

    let plan = build_plan(&medium, &options);

    if options.check {
        return check_update(&medium, &plan, options.json);
    }

    if options.dry_run {
        return print_plan(&medium, &plan, options.json);
    }

    execute(&medium, &plan, &options)?;
    if options.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": 1,
                "method": medium.name(),
                "package": medium.package(),
                "command": plan.display,
                "executed": true,
            }))
            .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
        );
    } else {
        println!("Update completed through {}.", medium.name());
    }
    Ok(())
}

fn parse_options(args: &[String]) -> anyhow::Result<Options> {
    let mut options = Options {
        check: false,
        dry_run: false,
        yes: false,
        force: false,
        all_features: false,
        json: false,
        version: None,
        method: MethodChoice::Auto,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => options.check = true,
            "--dry-run" => options.dry_run = true,
            "--yes" | "-y" => options.yes = true,
            "--force" => options.force = true,
            "--all-features" => options.all_features = true,
            "--json" => options.json = true,
            "--method" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(
                        help::UPDATE,
                        "--method needs auto, xpm, pacman, cargo or binary",
                    );
                };
                options.method = match value.as_str() {
                    "auto" => MethodChoice::Auto,
                    "xpm" => MethodChoice::Xpm,
                    "pacman" => MethodChoice::Pacman,
                    "cargo" => MethodChoice::Cargo,
                    "binary" | "bin" => MethodChoice::Binary,
                    other => help::usage_error(
                        help::UPDATE,
                        &format!(
                            "unknown method '{other}': use auto, xpm, pacman, cargo or binary"
                        ),
                    ),
                };
            }
            "--version" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::UPDATE, "--version needs a release version");
                };
                options.version = Some(value.trim_start_matches('v').to_string());
            }
            other => help::usage_error(help::UPDATE, &format!("unknown option '{other}'")),
        }
        index += 1;
    }
    Ok(options)
}

fn resolve_medium(choice: MethodChoice) -> anyhow::Result<Medium> {
    let exe = current_executable();
    match choice {
        MethodChoice::Xpm => xpm_medium(&exe).ok_or_else(|| {
            anyhow::anyhow!(
                "{} is not owned by an xpm package; use --method to select the correct medium",
                exe.display()
            )
        }),
        MethodChoice::Pacman => pacman_medium(&exe).ok_or_else(|| {
            anyhow::anyhow!(
                "{} is not owned by a pacman package; use --method to select the correct medium",
                exe.display()
            )
        }),
        MethodChoice::Cargo => Ok(Medium::Cargo),
        MethodChoice::Binary => Ok(Medium::Binary),
        MethodChoice::Auto => {
            if let Some(medium) = xpm_medium(&exe) {
                return Ok(medium);
            }
            if let Some(medium) = pacman_medium(&exe) {
                return Ok(medium);
            }
            if cargo_install_detected(&exe) {
                return Ok(Medium::Cargo);
            }
            Ok(Medium::Binary)
        }
    }
}

fn current_executable() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .or_else(|| std::env::current_exe().ok())
        .unwrap_or_else(|| PathBuf::from("xtop"))
}

fn package_owner(tool: &str, args: &[String]) -> Option<String> {
    let output = Command::new(tool).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())?
        .to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn xpm_medium(exe: &Path) -> Option<Medium> {
    let package = package_owner(
        "xpm",
        &[
            "query".into(),
            "--owner".into(),
            exe.display().to_string(),
            "-q".into(),
        ],
    )?;
    Some(Medium::Xpm { package })
}

fn pacman_medium(exe: &Path) -> Option<Medium> {
    let package = package_owner("pacman", &["-Qoq".into(), exe.display().to_string()])?;
    Some(Medium::Pacman { package })
}

fn cargo_install_detected(exe: &Path) -> bool {
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")));
    if cargo_home
        .as_ref()
        .map(|root| exe.starts_with(root.join("bin")))
        .unwrap_or(false)
    {
        return true;
    }
    Command::new("cargo")
        .args(["install", "--list"])
        .output()
        .map(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.trim_start().starts_with("xtop v"))
        })
        .unwrap_or(false)
}

fn build_plan(medium: &Medium, options: &Options) -> Plan {
    match medium {
        Medium::Xpm { package } => {
            let mut args = vec!["install".to_string()];
            if options.yes {
                args.push("--no-confirm".into());
            }
            if !options.force {
                args.push("--needed".into());
            }
            args.push(package.clone());
            let display = format!("xpm {}", args.join(" "));
            Plan {
                program: Some("xpm".into()),
                args,
                display,
                binary_script: None,
                needs_privilege: true,
            }
        }
        Medium::Pacman { package } => {
            let mut args = vec!["-S".to_string()];
            if options.yes {
                args.push("--noconfirm".into());
            }
            if !options.force {
                args.push("--needed".into());
            }
            args.push(package.clone());
            let display = format!("pacman {}", args.join(" "));
            Plan {
                program: Some("pacman".into()),
                args,
                display,
                binary_script: None,
                needs_privilege: true,
            }
        }
        Medium::Cargo => {
            let mut args = vec![
                "install".to_string(),
                "--git".into(),
                REPO.into(),
                "--locked".into(),
                "--force".into(),
            ];
            if options.all_features {
                args.push("--all-features".into());
            }
            if let Some(version) = &options.version {
                args.push("--tag".into());
                args.push(format!("v{version}"));
            }
            let display = format!("cargo {}", args.join(" "));
            Plan {
                program: Some("cargo".into()),
                args,
                display,
                binary_script: None,
                needs_privilege: false,
            }
        }
        Medium::Binary => {
            let script = if cfg!(windows) {
                POWERSHELL_INSTALLER
            } else {
                SHELL_INSTALLER
            };
            Plan {
                program: None,
                args: Vec::new(),
                display: format!(
                    "download and run {script} for {}",
                    current_executable().display()
                ),
                binary_script: Some(script),
                needs_privilege: !current_bin_dir_writable(),
            }
        }
    }
}

fn check_update(medium: &Medium, plan: &Plan, json_output: bool) -> anyhow::Result<()> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let latest = latest_version()?;
    let update_available = latest
        .as_deref()
        .map(|latest| is_newer(latest, &current))
        .unwrap_or(false);
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": 1,
                "method": medium.name(),
                "package": medium.package(),
                "installed_version": current,
                "latest_version": latest,
                "update_available": update_available,
                "command": plan.display,
                "executed": false,
            }))
            .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
        );
    } else {
        println!("method: {}", medium.name());
        if let Some(package) = medium.package() {
            println!("package: {package}");
        }
        println!("installed: {current}");
        match &latest {
            Some(latest) => println!("latest: {latest}"),
            None => println!("latest: no published release found"),
        }
        if update_available {
            println!("update available");
        } else {
            println!("up to date");
        }
        println!("command: {}", plan.display);
    }
    Ok(())
}

fn print_plan(medium: &Medium, plan: &Plan, json_output: bool) -> anyhow::Result<()> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schema": 1,
                "method": medium.name(),
                "package": medium.package(),
                "command": plan.display,
                "executed": false,
            }))
            .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
        );
    } else {
        println!("method: {}", medium.name());
        if let Some(package) = medium.package() {
            println!("package: {package}");
        }
        println!("command: {}", plan.display);
        println!("dry-run: not executed");
    }
    Ok(())
}

fn execute(medium: &Medium, plan: &Plan, options: &Options) -> anyhow::Result<()> {
    match medium {
        Medium::Binary => execute_binary(plan, options),
        Medium::Xpm { .. } | Medium::Pacman { .. } | Medium::Cargo => {
            let Some(program) = &plan.program else {
                anyhow::bail!("internal error: missing update program");
            };
            println!("Running: {}", plan.display);
            run_command(program, &plan.args, options.yes, plan.needs_privilege)
        }
    }
}

fn execute_binary(plan: &Plan, options: &Options) -> anyhow::Result<()> {
    let Some(url) = plan.binary_script else {
        anyhow::bail!("internal error: missing binary installer URL");
    };
    let bytes = assets::fetch_url(url)?;
    let extension = if cfg!(windows) { "ps1" } else { "sh" };
    let script =
        std::env::temp_dir().join(format!("xtop-update-{}.{extension}", std::process::id()));
    std::fs::write(&script, bytes)
        .map_err(|error| anyhow::anyhow!("failed to write {}: {error}", script.display()))?;
    let bin_dir = current_executable()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let (program, mut args) = if cfg!(windows) {
        (
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-File".to_string(),
                script.display().to_string(),
                "-BinDir".to_string(),
                bin_dir.display().to_string(),
            ],
        )
    } else {
        (
            "bash".to_string(),
            vec![
                script.display().to_string(),
                "--bin-dir".to_string(),
                bin_dir.display().to_string(),
                "--yes".to_string(),
            ],
        )
    };
    if let Some(version) = &options.version {
        args.push(if cfg!(windows) {
            "-Version".into()
        } else {
            "--version".into()
        });
        args.push(version.clone());
    }
    println!("Running: {} {}", program, args.join(" "));
    let result = run_command(&program, &args, options.yes, plan.needs_privilege);
    let _ = std::fs::remove_file(&script);
    result
}

fn run_command(program: &str, args: &[String], yes: bool, privileged: bool) -> anyhow::Result<()> {
    let mut command = if privileged && !is_root() {
        let mut sudo = Command::new("sudo");
        if yes {
            sudo.arg("-n");
        }
        sudo.arg("--").arg(program);
        sudo
    } else {
        Command::new(program)
    };
    command.args(args);
    let status = command
        .status()
        .map_err(|error| anyhow::anyhow!("failed to run {program}: {error}"))?;
    if !status.success() {
        anyhow::bail!("{program} exited with {status}");
    }
    Ok(())
}

#[cfg(unix)]
fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions and only reads process credentials.
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(unix))]
fn is_root() -> bool {
    false
}

fn current_bin_dir_writable() -> bool {
    let Some(parent) = current_executable().parent().map(Path::to_path_buf) else {
        return false;
    };
    let probe = parent.join(format!(".xtop-update-{}", std::process::id()));
    std::fs::write(&probe, b"ok")
        .and_then(|_| std::fs::remove_file(&probe))
        .is_ok()
}

fn latest_version() -> anyhow::Result<Option<String>> {
    let bytes = match assets::fetch_url(RELEASE_API) {
        Ok(bytes) => bytes,
        Err(error) if error.to_string().contains("404") => return Ok(None),
        Err(error) => return Err(error),
    };
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| anyhow::anyhow!("invalid GitHub release response: {error}"))?;
    let tag = value
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("GitHub release response has no tag_name"))?;
    Ok(Some(tag.trim_start_matches('v').to_string()))
}

fn is_newer(latest: &str, current: &str) -> bool {
    version_parts(latest) > version_parts(current)
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .trim_start_matches('v')
        .split(['-', '+'])
        .next()
        .unwrap_or(version)
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison_handles_release_tags() {
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
    }

    #[test]
    fn method_parser_accepts_all_names() {
        for value in ["auto", "xpm", "pacman", "cargo", "binary"] {
            let options = parse_options(&["--method".into(), value.into()]).expect("options");
            match value {
                "auto" => assert_eq!(options.method, MethodChoice::Auto),
                "xpm" => assert_eq!(options.method, MethodChoice::Xpm),
                "pacman" => assert_eq!(options.method, MethodChoice::Pacman),
                "cargo" => assert_eq!(options.method, MethodChoice::Cargo),
                "binary" => assert_eq!(options.method, MethodChoice::Binary),
                _ => unreachable!(),
            }
        }
    }
}
