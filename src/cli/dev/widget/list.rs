//! `xtop widget list`: show which widget packs are wired into the kernel.
//!
//! A pack counts as installed when it has a row in the compile-time pack
//! catalog (`tui/layout/pack_table.rs`, the engine's single source of truth)
//! *and* its Cargo feature is declared in the kernel `Cargo.toml` — the same
//! wiring check `xtop plugin list` performs for plugins. Rows without a
//! matching feature are leftovers (e.g. the feature was removed by hand) and
//! are skipped.

use std::fs;

use super::repo_root;

pub(crate) fn cmd_widget_list() {
    let root = repo_root();
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("kernel Cargo.toml should exist");
    let table = fs::read_to_string(root.join("src/tui/layout/pack_table.rs")).unwrap_or_default();

    let features = cargo_widget_features(&manifest);
    let rows = table_rows(&table);
    let installed: Vec<(&str, &str)> = rows
        .iter()
        .filter(|(feature, _)| feature == &"default" || features.iter().any(|f| f == feature))
        .map(|(f, l)| (*f, *l))
        .collect();

    if installed.is_empty() {
        println!("No widget packs installed.");
        println!("  Create one with `xtop widget scaffold <name>`, then wire it with");
        println!("  `xtop widget install <repo|path>`.");
        return;
    }
    println!("Widget packs wired into the kernel (Cargo.toml features + pack table):");
    for (feature, label) in installed {
        if feature == "default" {
            println!("  {label} (base pack, always on)");
        } else {
            println!("  {label} (feature: {feature})");
        }
    }
}

/// `widget-*` feature names declared in the kernel `Cargo.toml` that enable a
/// `xtop-widget-*` dependency (`widget-blocks = ["dep:xtop-widget-blocks"]`).
fn cargo_widget_features(manifest: &str) -> Vec<String> {
    manifest
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("widget-") && l.contains("dep:xtop-widget-"))
        .filter_map(|l| l.split('=').next().map(|f| f.trim().to_string()))
        .collect()
}

/// `(feature, label)` rows of the pack catalog file, in table order.
///
/// Handles both the single-line (`PackRow { feature: "…", label: "…" },`)
/// and the rustfmt multi-line spelling, and skips non-row lines.
fn table_rows(table: &str) -> Vec<(&str, &str)> {
    let mut rows = Vec::new();
    let mut in_row = false;
    let mut feature: Option<&str> = None;
    let mut label: Option<&str> = None;
    for line in table.lines() {
        let l = line.trim();
        if !in_row {
            if !l.starts_with("PackRow {") {
                continue;
            }
            in_row = true;
        }
        for (key, slot) in [("feature: \"", &mut feature), ("label: \"", &mut label)] {
            if let Some(idx) = l.find(key) {
                let rest = &l[idx + key.len()..];
                if let Some(end) = rest.find('"') {
                    *slot = Some(&rest[..end]);
                }
            }
        }
        if l.ends_with("},") && feature.is_some() && label.is_some() {
            rows.push((feature.unwrap(), label.unwrap()));
            in_row = false;
            feature = None;
            label = None;
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> &'static str {
        "[dependencies]\n\
         xtop-plugin-api = { git = \"https://github.com/xtop-cli/api\" }\n\
         xtop-widgets = { git = \"https://github.com/xtop-cli/widgets\" }\n\
         xtop-widget-blocks = { git = \"https://github.com/xtop-cli/widgets\", optional = true }\n\
         \n\
         [features]\n\
         default = [\"plugin-samurai\", \"mcp-extension\"]\n\
         plugin-samurai = [\"dep:xtop-plugin-samurai\"]\n\
         widget-blocks = [\"dep:xtop-widget-blocks\"]\n\
         widget-spark = [\"dep:xtop-widget-spark\"]\n"
    }

    fn sample_table() -> &'static str {
        "pub(crate) const PACK_TABLE: &[PackRow] = &[\n\
             PackRow { feature: \"default\", label: \"default\" },\n\
             // (xtop widget install appends installed-pack rows above this line.)\n\
             PackRow { feature: \"widget-blocks\", label: \"blocks\" },\n\
         ];\n"
    }

    #[test]
    fn parses_widget_feature_entries_from_the_manifest() {
        let features = cargo_widget_features(sample_manifest());
        assert_eq!(features, ["widget-blocks", "widget-spark"]);
    }

    #[test]
    fn parses_catalog_rows_in_table_order() {
        let rows = table_rows(sample_table());
        assert_eq!(rows, [("default", "default"), ("widget-blocks", "blocks"),]);
    }

    #[test]
    fn parses_rustfmt_multiline_rows() {
        // The live pack table is rustfmt-formatted: fields on their own
        // lines, closing `},` alone. Both spellings must parse identically.
        let table = "pub(crate) const PACK_TABLE: &[PackRow] = &[\n    PackRow {\n        feature: \"default\",\n        label: \"default\",\n    },\n    PackRow {\n        feature: \"widget-blocks\",\n        label: \"blocks\",\n    },\n];\n";
        assert_eq!(
            table_rows(table),
            [("default", "default"), ("widget-blocks", "blocks")]
        );
    }

    #[test]
    fn ignores_catalog_rows_without_matching_features() {
        let manifest =
            sample_manifest().replace("widget-spark = [\"dep:xtop-widget-spark\"]\n", "");
        let features = cargo_widget_features(&manifest);
        let installed: Vec<(&str, &str)> = table_rows(sample_table())
            .iter()
            .filter(|(f, _)| f == &"default" || features.iter().any(|x| x == f))
            .map(|(f, l)| (*f, *l))
            .collect();
        assert_eq!(
            installed,
            [("default", "default"), ("widget-blocks", "blocks")]
        );
    }
}
