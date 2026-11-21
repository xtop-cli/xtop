//! CLI compatibility, help and asset-management smoke tests.

use std::fs;
use std::process::Command;

fn xtop() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtop"))
}

fn help(args: &[&str]) -> std::process::Output {
    xtop().args(args).output().expect("run xtop help")
}

#[test]
fn global_help_lists_only_top_level_commands() {
    let out = help(&["--help"]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("xtop - cross-platform TUI system monitor"));
    for command in [
        "status [section]",
        "processes [options]",
        "watch [options]",
        "info",
        "doctor",
        "completions <shell>",
        "update [options]",
        "theme",
        "layout",
        "addon",
        "config",
        "plugin",
        "widget",
        "mcp",
        "--change-theme <theme>",
        "--gen-config",
        "--version",
        "-h, --help",
    ] {
        assert!(stdout.contains(command), "missing help entry: {command}");
    }
    for nested in [
        "theme list",
        "theme download",
        "layout download",
        "config download",
        "addon list",
        "plugin install",
        "widget install",
    ] {
        assert!(
            !stdout.contains(nested),
            "global help repeats nested command: {nested}"
        );
    }
}

#[test]
fn status_help_lists_sections_without_global_commands() {
    let status = help(&["status", "--help"]);
    assert!(status.status.success());
    let status = String::from_utf8_lossy(&status.stdout);
    assert!(status.starts_with("xtop status - "));
    assert!(status.contains("xtop status [section] [--json]"));
    assert!(status.contains("memory"));
    assert!(status.contains("disk-io"));
    assert!(!status.contains("theme list"));
    assert!(!status.contains("config download"));

    let memory = help(&["status", "memory", "--help"]);
    assert!(memory.status.success());
    let memory = String::from_utf8_lossy(&memory.stdout);
    assert!(memory.starts_with("xtop status memory - "));
    assert!(memory.contains("Usage:\n  xtop status memory [--json]"));
    assert!(memory.contains("JSON key: memory."));
}

#[test]
fn status_sections_return_only_their_slice() {
    let memory = xtop()
        .args(["status", "memory", "--json"])
        .output()
        .expect("run status memory");
    assert!(memory.status.success());
    let value: serde_json::Value = serde_json::from_slice(&memory.stdout).expect("memory JSON");
    assert_eq!(value["schema"], 1);
    assert_eq!(value["kind"], "status");
    assert!(value["memory"].is_object());
    assert!(value.get("cpu").is_none());
    assert!(value.get("disks").is_none());
    assert!(value.get("processes").is_none());

    let cpu = xtop()
        .args(["status", "cpu", "--json"])
        .output()
        .expect("run status cpu");
    assert!(cpu.status.success());
    let value: serde_json::Value = serde_json::from_slice(&cpu.stdout).expect("cpu JSON");
    assert!(value["cpu"].is_object());
    assert!(value.get("memory").is_none());

    let human = xtop()
        .args(["status", "memory"])
        .output()
        .expect("run status memory human");
    assert!(human.status.success());
    let human = String::from_utf8_lossy(&human.stdout);
    assert!(human.starts_with("memory: "));
    assert!(!human.contains("cpu: "));
}

#[test]
fn group_help_is_scoped_to_the_group() {
    let theme = help(&["theme", "--help"]);
    assert!(theme.status.success());
    let theme = String::from_utf8_lossy(&theme.stdout);
    assert!(theme.starts_with("xtop theme - "));
    assert!(theme.contains("download <theme|--all>"));
    assert!(!theme.contains("layout download"));

    let layout = help(&["layout", "--help"]);
    assert!(layout.status.success());
    let layout = String::from_utf8_lossy(&layout.stdout);
    assert!(layout.starts_with("xtop layout - "));
    assert!(layout.contains("check <file>"));

    let config = help(&["config", "--help"]);
    assert!(config.status.success());
    let config = String::from_utf8_lossy(&config.stdout);
    assert!(config.contains("download <name|--all>"));
    assert!(config.contains("get <key.path>"));
    assert!(config.contains("set <key.path> <value>"));

    let addon = help(&["addon", "--help"]);
    assert!(addon.status.success());
    assert!(String::from_utf8_lossy(&addon.stdout).contains("install <name|--all>"));

    let plural = help(&["layouts", "--help"]);
    assert!(plural.status.success());
    assert!(String::from_utf8_lossy(&plural.stdout).contains("xtop layout - "));

    let plugin = help(&["plugin", "--help"]);
    assert!(plugin.status.success());
    assert!(String::from_utf8_lossy(&plugin.stdout).contains("install <name|url>"));
}

#[test]
fn subcommand_help_lists_its_own_arguments() {
    for (args, expected) in [
        (vec!["theme", "set", "--help"], "xtop theme set <theme>"),
        (
            vec!["theme", "download", "--help"],
            "xtop theme download <theme>",
        ),
        (vec!["layout", "set", "--help"], "xtop layout set <layout>"),
        (
            vec!["layout", "download", "--help"],
            "xtop layout download <layout>",
        ),
        (
            vec!["layout", "check", "--help"],
            "xtop layout check <file.jsonc|file.json>",
        ),
        (
            vec!["addon", "install", "--help"],
            "xtop addon install <name>",
        ),
        (
            vec!["config", "download", "--help"],
            "xtop config download <name>",
        ),
        (
            vec!["config", "get", "--help"],
            "xtop config get <key.path>",
        ),
        (
            vec!["config", "set", "--help"],
            "xtop config set <key.path> <value>",
        ),
        (vec!["theme", "show", "--help"], "xtop theme show <theme>"),
        (
            vec!["layout", "show", "--help"],
            "xtop layout show <layout>",
        ),
        (vec!["status", "--help"], "xtop status"),
        (vec!["processes", "--help"], "xtop processes"),
        (vec!["watch", "--help"], "xtop watch"),
        (vec!["info", "--help"], "xtop info"),
        (vec!["doctor", "--help"], "xtop doctor"),
        (vec!["completions", "--help"], "xtop completions"),
        (vec!["update", "--help"], "xtop update"),
        (
            vec!["plugin", "install", "--help"],
            "xtop plugin install <name|url>",
        ),
    ] {
        let out = help(&args);
        assert!(out.status.success(), "help failed for {args:?}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(expected),
            "missing '{expected}' in help for {args:?}"
        );
    }

    let mcp = help(&["mcp", "--help"]);
    assert!(mcp.status.success());
    assert!(String::from_utf8_lossy(&mcp.stdout).contains("xtop mcp"));

    let theme = help(&["--change-theme", "--help"]);
    assert!(theme.status.success());
    assert!(String::from_utf8_lossy(&theme.stdout).contains("xtop --change-theme <theme>"));

    let gen = help(&["--gen-config", "--help"]);
    assert!(gen.status.success());
    assert!(String::from_utf8_lossy(&gen.stdout).contains("xtop --gen-config"));
}

#[test]
fn tui_requires_a_terminal_without_emitting_screen_sequences() {
    let out = xtop().output().expect("run xtop without a terminal");
    assert!(!out.status.success());
    assert!(
        out.stdout.is_empty(),
        "non-TTY startup must not emit alternate-screen/mouse sequences"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("requires a terminal"));
}

#[test]
fn unknown_flags_are_usage_errors_not_tui() {
    for flag in ["--ram", "-x"] {
        let out = xtop().arg(flag).output().expect("run unknown flag");
        assert_eq!(out.status.code(), Some(2), "flag {flag} must fail");
        assert!(
            out.stdout.is_empty(),
            "unknown flag {flag} must not enter the TUI"
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("unknown option"));
    }
}

#[test]
fn usage_errors_exit_with_status_two() {
    for args in [
        vec!["plugin", "install"],
        vec!["widget", "scaffold"],
        vec!["layout", "check"],
        vec!["layout", "unknown"],
        vec!["theme", "set"],
        vec!["theme", "download"],
        vec!["layout", "set"],
        vec!["config", "download"],
        vec!["--change-theme"],
        vec!["--gen-config", "extra"],
        vec!["--ram"],
        vec!["-x"],
        vec!["status", "--bogus"],
        vec!["status", "unknown"],
        vec!["status", "memory", "cpu"],
        vec!["completions", "unknown"],
        vec!["update", "--bogus"],
        vec!["update", "--method", "unknown"],
    ] {
        let out = xtop().args(&args).output().expect("run usage error");
        assert_eq!(
            out.status.code(),
            Some(2),
            "expected usage error for {args:?}"
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn gen_config_creates_base_assets() {
    let dir = std::env::temp_dir().join(format!("xtop-gen-config-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let out = xtop()
        .env("XDG_CONFIG_HOME", &dir)
        .arg("--gen-config")
        .output()
        .expect("run --gen-config");
    assert!(out.status.success());
    assert!(dir.join("xtop/config.json").is_file());
    assert!(dir.join("xtop/themes/x.jsonc").is_file());
    assert!(dir.join("xtop/layouts/dashboard.jsonc").is_file());
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(target_os = "linux")]
#[test]
fn config_download_installs_theme_from_a_local_catalog() {
    use sha2::{Digest, Sha256};

    let dir = std::env::temp_dir().join(format!("xtop-download-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create test dir");

    let palette = r##"["#000000","#111111","#222222","#333333","#444444","#555555","#666666","#777777","#888888","#999999","#aaaaaa","#bbbbbb","#cccccc","#dddddd","#eeeeee","#ffffff"]"##;
    let theme = format!(
        r##"{{"name":"test-theme","background":"#000000","foreground":"#ffffff","palette":{palette}}}"##
    );
    let theme_path = dir.join("test-theme.jsonc");
    fs::write(&theme_path, &theme).expect("write theme");
    let sha = format!("{:x}", Sha256::digest(theme.as_bytes()));

    let layout = r#"{"name":"Test Layout","root":{"direction":"vertical","areas":[{"widget":"cpu","size":"*"}]}}"#;
    let layout_path = dir.join("test-layout.jsonc");
    fs::write(&layout_path, layout).expect("write layout");
    let layout_sha = format!("{:x}", Sha256::digest(layout.as_bytes()));

    let catalog = format!(
        r#"{{
  "schema": 1,
  "catalog_version": "1.0.0",
  "generated": "2026-11-20",
  "base": {{ "theme": "x", "layout": "dashboard" }},
  "themes": [
    {{
      "id": "test-theme",
      "name": "test-theme",
      "version": "1.0.0",
      "author": "Test",
      "license": "GPL-3.0-or-later",
      "file": "test-theme.jsonc",
      "url": "{}",
      "sha256": "{}"
    }},
    {{
      "id": "x",
      "name": "x",
      "version": "1.0.0",
      "author": "Xscriptor",
      "license": "GPL-3.0-or-later",
      "file": "x.jsonc",
      "url": "{}",
      "sha256": ""
    }}
  ],
  "layouts": [
    {{
      "id": "dashboard",
      "name": "Dashboard",
      "category": "mode",
      "version": "1.0.0",
      "author": "Xscriptor",
      "license": "GPL-3.0-or-later",
      "file": "dashboard.jsonc",
      "url": "{}",
      "sha256": ""
    }},
    {{
      "id": "test-layout",
      "name": "Test Layout",
      "category": "custom",
      "version": "1.0.0",
      "author": "Test",
      "license": "GPL-3.0-or-later",
      "file": "test-layout.jsonc",
      "url": "{}",
      "sha256": "{}"
    }}
  ]
}}"#,
        theme_path.display(),
        sha,
        theme_path.display(),
        theme_path.display(),
        layout_path.display(),
        layout_sha
    );
    let catalog_path = dir.join("catalog.json");
    fs::write(&catalog_path, catalog).expect("write catalog");

    let config_home = dir.join("config");
    let download = xtop()
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XTOP_CATALOG_URL", &catalog_path)
        .args(["theme", "download", "test-theme"])
        .output()
        .expect("run theme download");
    assert!(
        download.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&download.stderr)
    );
    assert!(config_home.join("xtop/themes/test-theme.jsonc").is_file());
    assert!(config_home.join("xtop/assets.json").is_file());

    let set = xtop()
        .env("XDG_CONFIG_HOME", &config_home)
        .args(["theme", "set", "test-theme"])
        .output()
        .expect("run theme set");
    assert!(
        set.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&set.stderr)
    );
    let config = fs::read_to_string(config_home.join("xtop/config.json")).expect("read config");
    assert!(config.contains("\"theme\": \"test-theme\""));

    let layout_download = xtop()
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XTOP_CATALOG_URL", &catalog_path)
        .args(["layout", "download", "Test Layout"])
        .output()
        .expect("run layout download");
    assert!(
        layout_download.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&layout_download.stderr)
    );
    assert!(config_home.join("xtop/layouts/test-layout.jsonc").is_file());

    let layout_set = xtop()
        .env("XDG_CONFIG_HOME", &config_home)
        .args(["layout", "set", "Test Layout"])
        .output()
        .expect("run layout set");
    assert!(
        layout_set.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&layout_set.stderr)
    );
    let config = fs::read_to_string(config_home.join("xtop/config.json")).expect("read config");
    assert!(config.contains("\"layout_name\": \"Test Layout\""));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn version_prints_the_package_version() {
    let out = xtop().arg("--version").output().expect("run --version");
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("xtop {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn status_and_processes_produce_json_and_human_output() {
    let status = xtop()
        .args(["status", "--json"])
        .output()
        .expect("run status");
    assert!(status.status.success());
    let value: serde_json::Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(value["schema"], 1);
    assert_eq!(value["kind"], "status");
    assert!(value["host"].is_object());
    assert!(value["cpu"].is_object());
    assert!(value["memory"].is_object());

    let processes = xtop()
        .args([
            "processes",
            "--json",
            "--limit",
            "3",
            "--sort",
            "pid",
            "--asc",
        ])
        .output()
        .expect("run processes");
    assert!(processes.status.success());
    let value: serde_json::Value =
        serde_json::from_slice(&processes.stdout).expect("processes JSON");
    assert_eq!(value["kind"], "processes");
    let list = value["processes"].as_array().expect("process array");
    assert!(list.len() <= 3);
    if let Some(process) = list.first() {
        assert!(process.get("environ").is_none());
        assert!(process.get("pid").is_some());
    }

    let human = xtop()
        .args(["processes", "--limit", "1"])
        .output()
        .expect("run human processes");
    assert!(human.status.success());
    assert!(String::from_utf8_lossy(&human.stdout).contains("PID"));
}

#[test]
fn watch_can_emit_one_json_sample() {
    let out = xtop()
        .args(["watch", "--count", "1", "--jsonl"])
        .output()
        .expect("run watch");
    assert!(out.status.success());
    let line = String::from_utf8_lossy(&out.stdout);
    let value: serde_json::Value = serde_json::from_str(line.trim()).expect("watch JSONL");
    assert_eq!(value["kind"], "status");
}

#[test]
fn info_and_doctor_report_the_local_installation() {
    let dir = std::env::temp_dir().join(format!("xtop-info-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let info = xtop()
        .env("XDG_CONFIG_HOME", &dir)
        .args(["info", "--json"])
        .output()
        .expect("run info");
    assert!(info.status.success());
    let value: serde_json::Value = serde_json::from_slice(&info.stdout).expect("info JSON");
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert!(value["paths"].is_object());
    assert!(value["features"].is_array());

    let doctor = xtop()
        .env("XDG_CONFIG_HOME", &dir)
        .args(["doctor", "--json"])
        .output()
        .expect("run doctor");
    assert!(doctor.status.success());
    let value: serde_json::Value = serde_json::from_slice(&doctor.stdout).expect("doctor JSON");
    assert_eq!(value["ok"], true);
    assert!(value["checks"].is_array());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn config_get_set_list_and_reset_work() {
    let dir = std::env::temp_dir().join(format!("xtop-config-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let run = |args: &[&str]| {
        xtop()
            .env("XDG_CONFIG_HOME", &dir)
            .args(args)
            .output()
            .expect("run config command")
    };
    assert!(run(&["config", "set", "theme", "x"]).status.success());
    assert!(run(&["config", "set", "alerts.cpu_high", "80"])
        .status
        .success());
    let get = run(&["config", "get", "alerts.cpu_high"]);
    assert!(get.status.success());
    assert_eq!(String::from_utf8_lossy(&get.stdout).trim(), "80.0");
    let list = run(&["config", "list"]);
    assert!(list.status.success());
    let value: serde_json::Value = serde_json::from_slice(&list.stdout).expect("config JSON");
    assert_eq!(value["theme"], "x");
    assert_eq!(value["alerts"]["cpu_high"], 80.0);
    let reset = run(&["config", "reset", "--yes"]);
    assert!(reset.status.success());
    let get = run(&["config", "get", "alerts.cpu_high"]);
    assert_eq!(String::from_utf8_lossy(&get.stdout).trim(), "90.0");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn theme_and_layout_show_print_the_installed_model() {
    let theme = xtop()
        .args(["theme", "show", "x"])
        .output()
        .expect("run theme show");
    assert!(theme.status.success());
    let theme = String::from_utf8_lossy(&theme.stdout);
    assert!(theme.contains("name: x"));
    assert!(theme.contains("\"name\": \"x\""));

    let layout = xtop()
        .args(["layout", "show", "Dashboard"])
        .output()
        .expect("run layout show");
    assert!(layout.status.success());
    let layout = String::from_utf8_lossy(&layout.stdout);
    assert!(layout.contains("name: Dashboard"));
    assert!(layout.contains("\"name\": \"Dashboard\""));
}

#[test]
fn addon_list_and_path_cover_external_assets() {
    let dir = std::env::temp_dir().join(format!("xtop-addon-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let list = xtop()
        .env("XDG_CONFIG_HOME", &dir)
        .args(["addon", "list"])
        .output()
        .expect("run addon list");
    assert!(list.status.success());
    let list = String::from_utf8_lossy(&list.stdout);
    assert!(list.contains("themes:"));
    assert!(list.contains("layouts:"));
    assert!(list.contains("Dashboard"));

    let path = xtop()
        .env("XDG_CONFIG_HOME", &dir)
        .args(["addon", "path"])
        .output()
        .expect("run addon path");
    assert!(path.status.success());
    let path = String::from_utf8_lossy(&path.stdout);
    assert!(path.contains("themes:"));
    assert!(path.contains("layouts:"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn update_dry_run_reports_medium_and_command() {
    for (method, expected) in [
        ("cargo", "cargo install --git"),
        ("binary", "install-prebuilt"),
    ] {
        let out = xtop()
            .args(["update", "--dry-run", "--method", method, "--json"])
            .output()
            .expect("run update dry-run");
        assert!(out.status.success(), "dry-run failed for {method}");
        let value: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("update dry-run JSON");
        assert_eq!(value["method"], method);
        assert_eq!(value["executed"], false);
        assert!(
            value["command"].as_str().unwrap_or("").contains(expected),
            "missing command for {method}: {value}"
        );
    }
}

#[cfg(unix)]
#[test]
fn update_detects_xpm_and_pacman_package_ownership() {
    use std::os::unix::fs::PermissionsExt;

    for (tool, package) in [("xpm", "fake-xpm"), ("pacman", "fake-pacman")] {
        let dir = std::env::temp_dir().join(format!("xtop-update-{}-{}", tool, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fake tool dir");
        let script = dir.join(tool);
        fs::write(&script, format!("#!/bin/sh\nprintf '%s\\n' '{package}'\n"))
            .expect("write fake tool");
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("chmod fake tool");

        let path = format!(
            "{}:{}",
            dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let out = xtop()
            .env("PATH", path)
            .args(["update", "--dry-run", "--json"])
            .output()
            .expect("run update detection");
        assert!(out.status.success(), "detection failed for {tool}");
        let value: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("update detection JSON");
        assert_eq!(value["method"], tool);
        assert_eq!(value["package"], package);
        assert!(
            value["command"].as_str().unwrap_or("").contains(package),
            "command missing package for {tool}: {value}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn completions_are_available_for_common_shells() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let out = xtop()
            .args(["completions", shell])
            .output()
            .expect("run completions");
        assert!(out.status.success(), "completions failed for {shell}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("xtop"),
            "completion output missing xtop for {shell}"
        );
    }
}

#[test]
fn plugin_and_widget_list_read_the_kernel_wiring() {
    let plugins = xtop()
        .args(["plugin", "list"])
        .output()
        .expect("run plugin list");
    assert!(plugins.status.success());
    assert!(String::from_utf8_lossy(&plugins.stdout).contains("xtop-plugin-samurai"));

    let widgets = xtop()
        .args(["widget", "list"])
        .output()
        .expect("run widget list");
    assert!(widgets.status.success());
    let widgets = String::from_utf8_lossy(&widgets.stdout);
    assert!(widgets.contains("default (base pack, always on)"));
    assert!(widgets.contains("blocks"));
}

#[test]
fn layout_check_validates_structure_and_exit_codes() {
    let dir = std::env::temp_dir().join(format!("xtop-layout-test-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let valid = dir.join("valid.jsonc");
    let invalid = dir.join("invalid.jsonc");
    fs::write(
        &valid,
        r#"{
            // a widget leaf is a valid root
            "name": "CLI Test",
            "root": { "widget": "cpu", "size": "*" }
        }"#,
    )
    .expect("write valid layout");
    fs::write(&invalid, "{ invalid }").expect("write invalid layout");

    let ok = xtop()
        .args(["layout", "check", valid.to_str().unwrap()])
        .output()
        .expect("run valid layout check");
    assert!(ok.status.success());
    assert!(String::from_utf8_lossy(&ok.stdout).contains("is valid"));

    let bad = xtop()
        .args(["layout", "check", invalid.to_str().unwrap()])
        .output()
        .expect("run layout check");
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("INVALID"));

    let _ = fs::remove_dir_all(&dir);
}
