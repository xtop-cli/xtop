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
fn global_help_lists_every_command_on_stdout() {
    let out = help(&["--help"]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("xtop - cross-platform TUI system monitor"));
    for command in [
        "mcp",
        "theme list",
        "theme download",
        "layout list",
        "layout set",
        "layout download",
        "config download",
        "config generate",
        "plugin install",
        "widget install",
        "--change-theme <theme>",
        "--gen-config",
        "-h, --help",
    ] {
        assert!(stdout.contains(command), "missing help entry: {command}");
    }
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
    assert!(String::from_utf8_lossy(&config.stdout).contains("download <name|--all>"));

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
            vec!["config", "download", "--help"],
            "xtop config download <name>",
        ),
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
