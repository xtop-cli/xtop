//! `xtop plugin install`: register a plugin from a repo as an optional git
//! dependency + feature flag in the kernel `Cargo.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use super::{repo_root, LOCAL_PLUGINS_MARKER, PLUGINS_REPO};

pub(crate) fn cmd_plugin_install(name_or_url: &str) -> anyhow::Result<()> {
    let tmp = std::env::temp_dir().join("xtop-plugin-install");

    // Resolve the plugin package and its source repo.
    let (pkg_name, source_repo) = if is_git_url(name_or_url) {
        let repo = name_or_url;
        clone_into_tmp(repo, &tmp)?;
        let stem = Path::new(repo)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("plugin")
            .trim_start_matches("xtop-plugin-");
        let (_, pkg) = find_plugin_crate(&tmp, stem)?;
        (pkg, repo)
    } else {
        let name = name_or_url.trim().trim_start_matches("xtop-plugin-");
        if !is_valid_crate_name(&format!("xtop-plugin-{name}"), "xtop-plugin-") {
            anyhow::bail!("invalid plugin name '{name}'");
        }
        clone_into_tmp(PLUGINS_REPO, &tmp)?;
        if !tmp
            .join("plugins")
            .join(format!("xtop-plugin-{name}"))
            .is_dir()
        {
            let _ = fs::remove_dir_all(&tmp);
            anyhow::bail!(
                "Plugin '{name}' not found in {PLUGINS_REPO}. \
                 Available plugins live under plugins/xtop-plugin-<name>."
            );
        }
        (format!("xtop-plugin-{name}"), PLUGINS_REPO)
    };
    let feat_name = pkg_name.replace('-', "_");

    // Register a git dependency + feature flag in the kernel Cargo.toml.
    let manifest_path = repo_root().join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path)?;
    let dep_line = format!(
        "{pkg_name} = {{ git = {}, optional = true }}",
        toml_quote(source_repo)
    );
    let feature_line = format!("{feat_name} = [\"dep:{pkg_name}\"]");
    let mut new_content = content.clone();

    if content
        .lines()
        .any(|l| l.trim().starts_with(&format!("{pkg_name} =")))
    {
        println!("{pkg_name} is already a dependency of the kernel.");
    } else {
        if !content.contains(LOCAL_PLUGINS_MARKER) {
            new_content = new_content.replace(
                "[dependencies]",
                &format!("[dependencies]\n{LOCAL_PLUGINS_MARKER}"),
            );
        }
        new_content = new_content.replace(
            LOCAL_PLUGINS_MARKER,
            &format!("{LOCAL_PLUGINS_MARKER}\n{dep_line}"),
        );
    }
    if !content.contains(&feature_line) {
        new_content = new_content.replacen("[features]", &format!("[features]\n{feature_line}"), 1);
    }
    fs::write(&manifest_path, new_content)?;

    // Cleanup and verify the manifest resolves.
    let _ = fs::remove_dir_all(&tmp);
    println!("Verifying with `cargo check` ...");
    let status = std::process::Command::new("cargo")
        .args(["check"])
        .current_dir(repo_root())
        .status()
        .map_err(|e| anyhow::anyhow!("cargo check failed: {e}"))?;
    if !status.success() {
        anyhow::bail!("`cargo check` failed; check the plugin's compatibility.");
    }

    println!();
    println!("Plugin '{pkg_name}' installed successfully.");
    println!("  Source: {source_repo}");
    println!("  Feature flag: {feat_name} (NOT enabled by default)");
    println!("  Enable it: add '{feat_name}' to the [features] default list");
    println!("  in {manifest_path:?} and rebuild.");
    Ok(())
}

fn is_git_url(s: &str) -> bool {
    if s.starts_with('-')
        || s.chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '"' || c == '\\')
    {
        return false;
    }
    if let Some(pos) = s.find("://") {
        let scheme = &s[..pos];
        let mut chars = scheme.chars();
        let starts_ok = chars.next().is_some_and(|c| c.is_ascii_alphabetic());
        let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        return starts_ok && rest_ok;
    }
    s.starts_with("github.com/") || s.starts_with("git@")
}

/// Quote one string as a TOML basic string, escaping quotes, backslashes and
/// control characters. Keeps git URLs and local paths from breaking out of
/// the generated `Cargo.toml` entry.
fn toml_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Cargo package names are ASCII alphanumerics plus `-`/`_`. Validating the
/// suffix keeps a hostile manifest from injecting TOML or Rust source.
fn is_valid_crate_name(name: &str, prefix: &str) -> bool {
    name.len() <= 64
        && name.starts_with(prefix)
        && name.strip_prefix(prefix).is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

fn clone_into_tmp(repo_url: &str, tmp: &Path) -> anyhow::Result<()> {
    let _ = fs::remove_dir_all(tmp);
    println!("Cloning {repo_url} ...");
    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--filter=blob:none",
            "--sparse",
            "--",
            repo_url,
            tmp.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run git: {e}"))?;
    if !status.success() {
        anyhow::bail!("git clone failed");
    }
    Ok(())
}

/// Resolve where the plugin crate lives inside a cloned repo and its package
/// name. Accepts: crate at repo root, or `plugins/xtop-plugin-<name>`,
/// `plugins/<name>` or `crates/<name>` subdirectories.
fn find_plugin_crate(tmp: &Path, name: &str) -> anyhow::Result<(PathBuf, String)> {
    if !name.is_empty() && !is_valid_crate_name(&format!("xtop-plugin-{name}"), "xtop-plugin-") {
        anyhow::bail!("invalid plugin name '{name}'");
    }
    let candidates = [
        tmp.join("plugins").join(format!("xtop-plugin-{name}")),
        tmp.join("plugins").join(name),
        tmp.join("crates").join(format!("xtop-plugin-{name}")),
        tmp.join("crates").join(name),
        tmp.to_path_buf(), // root crate last
    ];
    for cand in &candidates {
        let manifest = cand.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let content = fs::read_to_string(&manifest).unwrap_or_default();
        if let Ok(toml) = content.parse::<toml::Value>() {
            if let Some(pkg) = toml.get("package").and_then(|p| p.get("name")) {
                if let Some(pkg_name) = pkg.as_str() {
                    if is_valid_crate_name(pkg_name, "xtop-plugin-") {
                        println!("Found plugin at {}", cand.display());
                        return Ok((cand.to_path_buf(), pkg_name.to_string()));
                    }
                }
            }
        }
    }
    let _ = fs::remove_dir_all(tmp);
    anyhow::bail!(
        "No `xtop-plugin-*` package found for '{name}' in {}. \
         URL installs need the crate at the repo root or in plugins//crates/.",
        tmp.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_option_injection_and_bad_names() {
        assert!(is_git_url("https://github.com/xtop-cli/plugins"));
        assert!(is_git_url("git@github.com:me/xtop-plugin-x.git"));
        assert!(is_git_url("github.com/me/xtop-plugin-x"));
        assert!(!is_git_url("--upload-pack=evil"));
        assert!(!is_git_url("https://github.com/x y"));

        assert!(is_valid_crate_name("xtop-plugin-demo", "xtop-plugin-"));
        assert!(!is_valid_crate_name("xtop-plugin-", "xtop-plugin-"));
        assert!(!is_valid_crate_name(
            "xtop-plugin-x\"] [build-dependencies]",
            "xtop-plugin-"
        ));
        assert_eq!(
            toml_quote("https://github.com/xtop-cli/plugins"),
            "\"https://github.com/xtop-cli/plugins\""
        );
    }
}
