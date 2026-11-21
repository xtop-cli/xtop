//! Widget-pack management subcommands (list, install, scaffold).
//!
//! Widget packs live in their own crates (`xtop-widget-<name>` — the
//! installable/designable unit) and are integrated into the kernel exactly
//! like plugins: optional Cargo git/path dependencies + feature flags, plus
//! a row in the compile-time pack catalog (`tui/layout/pack_table.rs`) so the
//! render engine and `xtop widget list` enumerate one source of truth.
//! These commands edit the kernel's own sources accordingly; none of them
//! touch the user config or the runtime.

use std::path::Path;

use crate::cli::help;

mod install;
mod list;
mod scaffold;

pub const WIDGETS_REPO: &str = "https://github.com/xtop-cli/widgets";
pub(crate) const LOCAL_WIDGETS_MARKER: &str = "# Local widget pack installs (xtop widget install)";

/// Kernel repo root: `Cargo.toml` lives right at `CARGO_MANIFEST_DIR`.
pub(crate) fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The pack-catalog source file `xtop widget install` self-edits.
pub(crate) fn pack_table_path() -> std::path::PathBuf {
    repo_root().join("src/tui/layout/pack_table.rs")
}

/// Handle `xtop widget <sub>` from the parsed argument vector.
pub fn widget_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::WIDGET);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::WIDGET);
            Ok(())
        }
        "list" => {
            if help::wants_help(&args[3..]) {
                help::show(help::WIDGET_LIST);
                return Ok(());
            }
            list::cmd_widget_list();
            Ok(())
        }
        "install" => {
            if help::wants_help(&args[3..]) {
                help::show(help::WIDGET_INSTALL);
                return Ok(());
            }
            let Some(target) = args.get(3) else {
                help::usage_error(
                    help::WIDGET_INSTALL,
                    "Usage: xtop widget install <name|url|path>",
                );
            };
            install::cmd_widget_install(target)
        }
        "scaffold" => {
            if help::wants_help(&args[3..]) {
                help::show(help::WIDGET_SCAFFOLD);
                return Ok(());
            }
            let Some(name) = args.get(3) else {
                help::usage_error(help::WIDGET_SCAFFOLD, "Usage: xtop widget scaffold <name>");
            };
            scaffold::cmd_widget_scaffold(name)
        }
        other => help::usage_error(
            help::WIDGET,
            &format!("xtop widget: unknown command '{other}'"),
        ),
    }
}
