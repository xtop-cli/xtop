//! Plugin management subcommands (list, install, scaffold).
//!
//! Since the kernel became a monocrate, plugins live in their own repos
//! (`xtop-cli/plugins`) and are integrated through Cargo git dependencies +
//! feature flags, exactly like the built-in `xtop-plugin-samurai`. These
//! commands edit the kernel's own `Cargo.toml` accordingly.

use std::path::Path;

use crate::cli::help;

mod install;
mod list;
mod scaffold;

pub const PLUGINS_REPO: &str = "https://github.com/xtop-cli/plugins";
pub(crate) const LOCAL_PLUGINS_MARKER: &str = "# Local plugin installs (xtop plugin install)";

/// Kernel repo root: `Cargo.toml` lives right at `CARGO_MANIFEST_DIR`.
pub(crate) fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Handle `xtop plugin <sub>` from the parsed argument vector.
pub fn plugin_command(args: &[String]) -> anyhow::Result<()> {
    let Some(sub) = args.get(2) else {
        help::show(help::PLUGIN);
        return Ok(());
    };
    match sub.as_str() {
        "-h" | "--help" => {
            help::show(help::PLUGIN);
            Ok(())
        }
        "list" => {
            if help::wants_help(&args[3..]) {
                help::show(help::PLUGIN_LIST);
                return Ok(());
            }
            list::cmd_plugin_list();
            Ok(())
        }
        "install" => {
            if help::wants_help(&args[3..]) {
                help::show(help::PLUGIN_INSTALL);
                return Ok(());
            }
            let Some(target) = args.get(3) else {
                help::usage_error(
                    help::PLUGIN_INSTALL,
                    "Usage: xtop plugin install <name|url>",
                );
            };
            install::cmd_plugin_install(target)
        }
        "scaffold" => {
            if help::wants_help(&args[3..]) {
                help::show(help::PLUGIN_SCAFFOLD);
                return Ok(());
            }
            let Some(name) = args.get(3) else {
                help::usage_error(help::PLUGIN_SCAFFOLD, "Usage: xtop plugin scaffold <name>");
            };
            scaffold::cmd_plugin_scaffold(name)
        }
        other => help::usage_error(
            help::PLUGIN,
            &format!("xtop plugin: unknown command '{other}'"),
        ),
    }
}
