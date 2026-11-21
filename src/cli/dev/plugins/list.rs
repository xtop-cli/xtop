//! `xtop plugin list`: show which plugins are wired into the kernel.

use std::fs;

use super::repo_root;

pub(crate) fn cmd_plugin_list() {
    let manifest = fs::read_to_string(repo_root().join("Cargo.toml"))
        .expect("kernel Cargo.toml should exist next to the binary");

    // A plugin counts as "installed" when it has a matching `dep:<name>`
    // feature entry (like the built-in xtop-plugin-samurai). Contract crates
    // (xtop-plugin-api) are excluded.
    let in_features = manifest
        .lines()
        .skip_while(|l| !l.trim().starts_with("[features]"))
        .any(|l| l.contains("dep:"));
    if !in_features {
        println!("No plugins installed.");
        return;
    }

    let mut plugins: Vec<String> = Vec::new();
    for line in manifest.lines() {
        let l = line.trim();
        if !l.starts_with("xtop-plugin-") || !l.contains('=') || l.starts_with('#') {
            continue;
        }
        let name = l.split('=').next().unwrap_or("").trim().to_string();
        if !name.is_empty() {
            let dep_ref = format!("dep:{name}");
            if manifest.lines().any(|f| f.contains(&dep_ref)) {
                plugins.push(name);
            }
        }
    }

    if plugins.is_empty() {
        println!("No plugins installed.");
        return;
    }
    println!("Plugins wired into the kernel (Cargo.toml):");
    for p in plugins {
        println!("  {p}");
    }
}
