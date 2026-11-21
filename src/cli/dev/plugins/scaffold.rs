//! `xtop plugin scaffold`: generate a fresh plugin crate template under
//! `plugins-dev/` (ignored by git), ready to push as its own repo.

use std::fs;

use super::repo_root;

pub(crate) fn cmd_plugin_scaffold(name: &str) -> anyhow::Result<()> {
    let plugin_dir = repo_root()
        .join("plugins-dev")
        .join(format!("xtop-plugin-{name}"));
    if plugin_dir.exists() {
        anyhow::bail!("Plugin crate already exists at {}", plugin_dir.display());
    }
    let src_dir = plugin_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let cap: String = {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    };

    let cargo_toml = format!(
        r#"[package]
name = "xtop-plugin-{name}"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"
description = "xtop plugin: {name}"

[dependencies]
xtop-plugin-api = {{ git = "https://github.com/xtop-cli/api" }}
"#
    );
    fs::write(plugin_dir.join("Cargo.toml"), &cargo_toml)?;

    let lib_rs = format!(
        r#"//! {cap} plugin for xtop.
use xtop_plugin_api::{{Plugin, PluginCapability, PluginContext, PluginError, PluginManifest}};

pub struct {cap}Plugin;

impl {cap}Plugin {{
    pub fn new() -> Self {{
        Self
    }}
}}

impl Plugin for {cap}Plugin {{
    fn manifest(&self) -> PluginManifest {{
        PluginManifest {{
            id: "{name}".to_string(),
            name: "{cap}".to_string(),
            version: "0.1.0".to_string(),
            description: "xtop plugin: {name}".to_string(),
            capabilities: vec![PluginCapability::ReadSystemInfo],
        }}
    }}

    fn on_tick(&mut self, _ctx: &mut PluginContext) -> Result<(), PluginError> {{
        Ok(())
    }}
}}
"#,
        cap = cap,
        name = name
    );
    fs::write(src_dir.join("lib.rs"), &lib_rs)?;

    println!("Plugin scaffold created at {}", plugin_dir.display());
    println!("To integrate it into the kernel:");
    println!(
        "  1. `xtop plugin install {}` (works for repos with the crate",
        name
    );
    println!("     at the root or under plugins//crates/).");
    println!("  2. Implement the Plugin trait methods.");
    Ok(())
}
