//! `xtop widget install`: register a widget pack from a repo (or a local
//! path) as an optional Cargo dependency + feature flag and add it to the
//! compile-time pack catalog, so the engine can render it.
//!
//! Mirrors `xtop plugin install`: a self-modifying-source workflow that
//! edits the kernel's own files (root `Cargo.toml` + the pack catalog in
//! `src/ui/layout/pack_table.rs`) and runs `cargo check`. It never enables
//! the new feature by default — the user opts in by adding it to the
//! `[features]` default list (or building with `--features`) and rebuilding.

use std::fs;
use std::path::{Path, PathBuf};

use super::{pack_table_path, repo_root, LOCAL_WIDGETS_MARKER, WIDGETS_REPO};

/// Anchor comments the installer inserts rows/arms next to. The exact same
/// single-line comments live in `src/ui/layout/pack_table.rs`; keep the two
/// texts in sync (the pack-table module doc names this file as the owner —
/// the marker text deliberately never appears inside a string constant in
/// pack_table.rs itself, so a self-edit can never mangle its own anchor).
const TABLE_INSTALL_MARKER: &str =
    "// (xtop widget install appends installed-pack rows above this line.)";
const ARM_INSTALL_MARKER: &str =
    "// (xtop widget install appends installed-pack arms above this line.)";

pub(crate) fn cmd_widget_install(name_or_url: &str) -> anyhow::Result<()> {
    // Resolve the pack crate and its source (local dir, git URL, or a bare
    // name in the xtop-cli/widgets repo).
    let source = Source::resolve(name_or_url)?;
    let pkg = source.package_name()?;
    if !is_valid_crate_name(&pkg, "xtop-widget-") {
        let _ = source.cleanup();
        anyhow::bail!(
            "package '{pkg}' is not a widget pack: expected a name starting with `xtop-widget-`"
        );
    }
    let label = pkg
        .strip_prefix("xtop-widget-")
        .expect("validated widget package name");
    let feature = format!("widget-{label}");
    let crate_ident = pkg.replace('-', "_");

    // 1. Cargo.toml: optional dependency + feature flag (plugin pattern).
    edit_cargo_manifest(&pkg, &source, &feature)?;

    // 2. Pack catalog: one PACK_TABLE row + one cfg-gated registry arm.
    edit_pack_table(label, &feature, &crate_ident)?;

    // 3. Cleanup and verify the manifest resolves.
    let _ = source.cleanup();
    println!("Verifying with `cargo check` ...");
    let status = std::process::Command::new("cargo")
        .args(["check"])
        .current_dir(repo_root())
        .status()
        .map_err(|e| anyhow::anyhow!("cargo check failed: {e}"))?;
    if !status.success() {
        anyhow::bail!("`cargo check` failed; check the pack's compatibility.");
    }

    println!();
    println!("Widget pack '{pkg}' installed successfully.");
    println!("  Source: {}", source.display());
    println!("  Feature flag: {feature} (NOT enabled by default)");
    println!("  Pack label: {label} (use it in style.pack / style.widgets.<name>.pack)");
    println!("  Enable it: add '{feature}' to the [features] default list");
    println!("  in {:?} and rebuild.", repo_root().join("Cargo.toml"));
    Ok(())
}

/// A resolved pack source: a local directory or a git URL (with its temp
/// clone). URL clones are cleaned up after install; local paths stay.
enum Source {
    Local(PathBuf),
    Git { url: String, tmp: PathBuf },
}

impl Source {
    fn resolve(name_or_url: &str) -> anyhow::Result<Self> {
        let input = name_or_url.trim().trim_end_matches('/');
        if is_local_path(input) {
            let dir = expand_tilde(Path::new(input));
            if !dir.is_dir() {
                anyhow::bail!("path '{}' is not a directory", dir.display());
            }
            return Ok(Self::Local(dir));
        }
        if is_git_url(input) {
            let tmp = std::env::temp_dir().join("xtop-widget-install");
            clone_into_tmp(input, &tmp)?;
            return Ok(Self::Git {
                url: input.to_string(),
                tmp,
            });
        }
        // Bare name: look the pack up in the widgets repo, like plugins.
        let name = input.trim_start_matches("xtop-widget-");
        if !is_valid_crate_name(&format!("xtop-widget-{name}"), "xtop-widget-") {
            anyhow::bail!("invalid widget pack name '{name}'");
        }
        let tmp = std::env::temp_dir().join("xtop-widget-install");
        clone_into_tmp(WIDGETS_REPO, &tmp)?;
        let found = find_pack_crate(&tmp, name).is_ok();
        if !found {
            let _ = fs::remove_dir_all(&tmp);
            anyhow::bail!(
                "Widget pack '{name}' not found in {WIDGETS_REPO}. Packs live under \
                 packs/xtop-widget-<name>; or pass a git URL or a local path."
            );
        }
        Ok(Self::Git {
            url: WIDGETS_REPO.to_string(),
            tmp,
        })
    }

    fn display(&self) -> String {
        match self {
            Self::Local(dir) => dir.display().to_string(),
            Self::Git { url, .. } => url.clone(),
        }
    }

    /// The package name of the pack crate this source carries.
    fn package_name(&self) -> anyhow::Result<String> {
        let (root, hint) = match self {
            Self::Local(dir) => (dir.clone(), None),
            Self::Git { tmp, .. } => {
                let hint = Path::new(&self.display())
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("widget")
                    .trim_start_matches("xtop-widget-")
                    .to_string();
                (tmp.clone(), Some(hint))
            }
        };
        let hint = hint.unwrap_or_default();
        let (crate_dir, _) = find_pack_crate(&root, &hint)?;
        package_name_of(&crate_dir)
            .ok_or_else(|| anyhow::anyhow!("no parseable Cargo.toml at {}", crate_dir.display()))
    }

    fn cleanup(&self) -> anyhow::Result<()> {
        if let Self::Git { tmp, .. } = self {
            fs::remove_dir_all(tmp)?;
        }
        Ok(())
    }
}

/// Edit the kernel `Cargo.toml`: optional dep under the local-installs
/// marker + a feature line right after `[features]` (plugin install flow).
fn edit_cargo_manifest(pkg: &str, source: &Source, feature: &str) -> anyhow::Result<()> {
    let manifest_path = repo_root().join("Cargo.toml");
    let content = fs::read_to_string(&manifest_path)?;
    let dep_line = match source {
        Source::Local(dir) => format!(
            "{pkg} = {{ path = {}, optional = true }}",
            toml_quote(&dir.display().to_string())
        ),
        Source::Git { url, .. } => {
            format!("{pkg} = {{ git = {}, optional = true }}", toml_quote(url))
        }
    };
    let feature_line = format!("{feature} = [\"dep:{pkg}\"]");
    let mut new_content = content.clone();

    if content
        .lines()
        .any(|l| l.trim().starts_with(&format!("{pkg} =")))
    {
        println!("{pkg} is already a dependency of the kernel.");
    } else {
        if !content.contains(LOCAL_WIDGETS_MARKER) {
            new_content = new_content.replace(
                "[dependencies]",
                &format!("[dependencies]\n{LOCAL_WIDGETS_MARKER}"),
            );
        }
        new_content = new_content.replace(
            LOCAL_WIDGETS_MARKER,
            &format!("{LOCAL_WIDGETS_MARKER}\n{dep_line}"),
        );
    }
    if !content.contains(&feature_line) {
        new_content = new_content.replacen("[features]", &format!("[features]\n{feature_line}"), 1);
    }
    fs::write(&manifest_path, new_content)?;
    Ok(())
}

/// Edit `src/ui/layout/pack_table.rs`: one `PACK_TABLE` row at the table
/// marker and one cfg-gated match arm (carrying its registry static) at the
/// arm marker. Both insertions are text-identical in shape to the shipped
/// rows/arms, so the file stays rustfmt-clean.
fn edit_pack_table(label: &str, feature: &str, crate_ident: &str) -> anyhow::Result<()> {
    let path = pack_table_path();
    let content = fs::read_to_string(&path)?;
    if content.contains(&format!("feature: \"{feature}\"")) {
        println!("{feature} is already in the pack table.");
        return Ok(());
    }

    // The generated text is rustfmt-canonical (verified by the e2e gate):
    // rows and arms are inserted directly above their anchor comment, and
    // the anchor is rewritten with its canonical indentation.
    let row = format!(
        "    PackRow {{\n        feature: \"{feature}\",\n        label: \"{label}\",\n    }},\n"
    );
    let arm = format!(
        "        #[cfg(feature = \"{feature}\")]\n\
         \x20       \"{feature}\" => {{\n\
         \x20           static {crate_ident}_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> =\n\
         \x20               OnceLock::new();\n\
         \x20           v.push(Pack {{\n\
         \x20               name: row.label,\n\
         \x20               renderers: {crate_ident}_PACK.get_or_init({crate_ident}::registry),\n\
         \x20           }});\n\
         \x20       }}\n"
    );

    if !content.contains(TABLE_INSTALL_MARKER) || !content.contains(ARM_INSTALL_MARKER) {
        anyhow::bail!(
            "pack table markers not found in {}; refusing to edit a file I do not recognize",
            path.display()
        );
    }
    let mut new_content = content;
    // The anchors are replaced *with their canonical line indentation* (the
    // marker line is matched including the indent, so the inserted text is
    // not pushed right by the surrounding source indentation).
    new_content = new_content.replacen(
        &format!("    {TABLE_INSTALL_MARKER}"),
        &format!("{row}    {TABLE_INSTALL_MARKER}"),
        1,
    );
    new_content = new_content.replacen(
        &format!("        {ARM_INSTALL_MARKER}"),
        &format!("{arm}        {ARM_INSTALL_MARKER}"),
        1,
    );
    fs::write(&path, new_content)?;
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

fn is_local_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with('.') || s.starts_with('~')
}

fn expand_tilde(path: &Path) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(rest) = path.strip_prefix("~/") {
                return Path::new(&home).join(rest);
            }
        }
    }
    path.to_path_buf()
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

/// Resolve where the pack crate lives inside a root directory and validate
/// it. Accepts: crate at the root, `packs/xtop-widget-<name>`,
/// `packs/<name>`, `xtop-widget-<name>` and `widgets/xtop-widget-<name>`.
fn find_pack_crate(root: &Path, name: &str) -> anyhow::Result<(PathBuf, String)> {
    let named = format!("xtop-widget-{name}");
    if !name.is_empty() && !is_valid_crate_name(&named, "xtop-widget-") {
        anyhow::bail!("invalid widget pack name '{name}'");
    }
    let candidates = [
        root.join("packs").join(&named),
        root.join(&named),
        root.join("packs").join(name),
        root.join("widgets").join(&named),
        root.to_path_buf(), // root crate last
    ];
    for cand in &candidates {
        let manifest = cand.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let content = fs::read_to_string(&manifest).unwrap_or_default();
        if let Ok(toml) = content.parse::<toml::Value>() {
            if let Some(pkg_name) = toml
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
            {
                if is_valid_crate_name(pkg_name, "xtop-widget-") {
                    println!("Found widget pack at {}", cand.display());
                    return Ok((cand.to_path_buf(), pkg_name.to_string()));
                }
            }
        }
    }
    anyhow::bail!(
        "no `xtop-widget-*` package found for '{name}' in {}. \
         Repos need the crate at the root or under packs//widgets/.",
        root.display()
    )
}

/// The package name declared in the Cargo.toml of a crate directory.
fn package_name_of(crate_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(crate_dir.join("Cargo.toml")).ok()?;
    let toml = content.parse::<toml::Value>().ok()?;
    toml.get("package")?
        .get("name")?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_table_row_matches_the_shipped_shape() {
        // The row a fresh install inserts is byte-shaped like the shipped
        // rows, so the file stays rustfmt-clean after the self-edit.
        let row =
            "    PackRow {\n        feature: \"widget-demo\",\n        label: \"demo\",\n    },\n";
        let table = "pub(crate) const PACK_TABLE: &[PackRow] = &[\n    PackRow {\n        feature: \"default\",\n        label: \"default\",\n    },\n    // (xtop widget install appends installed-pack rows above this line.)\n];\n";
        let edited = table.replacen(
            TABLE_INSTALL_MARKER,
            &format!("{row}{TABLE_INSTALL_MARKER}"),
            1,
        );
        assert!(edited.contains("feature: \"widget-demo\""));
        assert_eq!(edited.matches(TABLE_INSTALL_MARKER).count(), 1);
        // The row lands above the marker, before the closing `];`.
        let marker_at = edited.find(TABLE_INSTALL_MARKER).unwrap();
        assert!(edited[..marker_at].contains("widget-demo"));
        assert!(edited[marker_at..].contains("];"));
    }

    #[test]
    fn generated_arm_is_self_contained_and_marker_stays() {
        let mut h = edit_helpers_fixture();
        let row =
            "    PackRow {\n        feature: \"widget-demo\",\n        label: \"demo\",\n    },\n"
                .to_string();
        let arm = "        #[cfg(feature = \"widget-demo\")]\n\
             \"widget-demo\" => {\n\
             \x20           static XTOP_WIDGET_DEMO_PACK: OnceLock<HashMap<&'static str, WidgetRenderer>> =\n\
             \x20               OnceLock::new();\n\
             \x20           v.push(Pack {\n\
             \x20               name: row.label,\n\
             \x20               renderers: XTOP_WIDGET_DEMO_PACK.get_or_init(xtop_widget_demo::registry),\n\
             \x20           });\n\
             \x20       },\n"
            .to_string();
        h = h.replacen(
            TABLE_INSTALL_MARKER,
            &format!("{row}{TABLE_INSTALL_MARKER}"),
            1,
        );
        h = h.replacen(ARM_INSTALL_MARKER, &format!("{arm}{ARM_INSTALL_MARKER}"), 1);
        // Both the row and the arm reference the same feature, and the arm
        // statics + registry reference survive in the text.
        assert_eq!(h.matches("feature: \"widget-demo\"").count(), 1);
        assert!(h.contains("XTOP_WIDGET_DEMO_PACK.get_or_init(xtop_widget_demo::registry)"));
        // Markers survive for repeated installs.
        assert!(h.contains(TABLE_INSTALL_MARKER));
        assert!(h.contains(ARM_INSTALL_MARKER));
    }

    #[test]
    fn url_and_path_detection() {
        assert!(is_git_url("https://github.com/xtop-cli/widgets"));
        assert!(is_git_url("git@github.com:me/xtop-widget-x.git"));
        assert!(is_git_url("github.com/xtop-cli/widgets"));
        assert!(!is_git_url("--upload-pack=evil"));
        assert!(!is_git_url("https://github.com/x y"));
        assert!(!is_git_url("https://github.com/x\"y"));
        assert!(is_local_path("/home/me/widgets"));
        assert!(is_local_path("./widgets-dev/xtop-widget-x"));
        assert!(is_local_path("~/code/xtop-widget-x"));
        assert!(!is_local_path("github.com/xtop-cli/widgets"));
    }

    #[test]
    fn crate_names_and_toml_quoting_are_safe() {
        assert!(is_valid_crate_name("xtop-widget-demo", "xtop-widget-"));
        assert!(is_valid_crate_name("xtop-widget-disk_io", "xtop-widget-"));
        assert!(!is_valid_crate_name("xtop-widget-", "xtop-widget-"));
        assert!(!is_valid_crate_name(
            "xtop-widget-x\"; evil = true",
            "xtop-widget-"
        ));
        assert_eq!(toml_quote("https://x/y"), "\"https://x/y\"");
        assert_eq!(toml_quote("C:\\x\\y"), "\"C:\\\\x\\\\y\"");
        assert_eq!(toml_quote("a\"b"), "\"a\\\"b\"");
    }

    /// A minimal pack_table.rs fixture with both markers present.
    fn edit_helpers_fixture() -> String {
        "//! Pack catalog fixture.\nuse std::collections::HashMap;\nuse std::sync::OnceLock;\n\npub(crate) const PACK_TABLE: &[PackRow] = &[\n    PackRow {\n        feature: \"default\",\n        label: \"default\",\n    },\n    // (xtop widget install appends installed-pack rows above this line.)\n    PackRow {\n        feature: \"widget-blocks\",\n        label: \"blocks\",\n    },\n];\n\nfn push_optional_pack(v: &mut Vec<Pack>, row: &PackRow) {\n    match row.feature {\n        // (xtop widget install appends installed-pack arms above this line.)\n        #[cfg(feature = \"widget-blocks\")]\n        \"widget-blocks\" => v.push(Pack {\n            name: row.label,\n            renderers: BLOCKS_PACK.get_or_init(xtop_widget_blocks::registry),\n        }),\n        _ => {}\n    }\n}\n"
            .to_string()
    }
}
