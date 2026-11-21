//! Help text for the hand-written CLI.
//!
//! The help is a tree: the global `--help` lists top-level commands and
//! groups, each group lists its own leaves, and each leaf prints only its
//! arguments and notes. `docs/commands.md` mirrors this content.

mod assets;
mod dev;
mod system;

pub(crate) use assets::*;
pub(crate) use dev::*;
pub(crate) use system::*;

pub(crate) const GLOBAL: &str = "\
xtop - cross-platform TUI system monitor and kernel CLI

Usage:
  xtop
  xtop <command> [arguments]
  xtop --change-theme <theme>
  xtop --gen-config
  xtop --help

Commands:
  status [section]        Snapshot or one section (cpu, memory, swap; status --help)
  processes [options]     Print a process list (processes --help)
  watch [options]         Print periodic snapshots (watch --help)
  info                    Show build, path and tool information
  doctor                  Diagnose the local installation
  completions <shell>     Print shell completions
  update [options]        Update the installation
  theme                   Manage themes (theme --help)
  layout                  Manage layouts (layout --help)
  addon                   Manage installable themes and layouts
  config                  Manage configuration and paths
  plugin                  Manage compiled plugins
  widget                  Manage compiled widget packs
  mcp                     Run the MCP server over stdio

Global options:
  --change-theme <theme>  Change the active theme and persist it
  --ct                    Deprecated alias for --change-theme
  --gen-config            Create config.json, themes/x.jsonc and layouts/dashboard.jsonc
  -V, --version           Print the version and exit
  -h, --help              Show this help and exit

Use 'xtop <command> --help' for the full command tree.
Without a command, xtop starts the interactive TUI.";

/// Print help text to stdout.
pub(crate) fn show(text: &str) {
    println!("{text}");
}

/// Print a usage error and the relevant help to stderr, then exit with
/// status 2. Usage errors are distinct from runtime failures (status 1).
pub(crate) fn usage_error(text: &str, message: &str) -> ! {
    eprintln!("{message}");
    eprintln!();
    eprintln!("{text}");
    std::process::exit(2);
}

/// Whether a subcommand argument list asks for help.
pub(crate) fn wants_help(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-h" || arg == "--help")
}
