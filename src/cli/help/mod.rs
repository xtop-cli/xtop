//! Help text for the hand-written CLI.
//!
//! The help is layered by command: the global `--help` lists every command
//! and each group/subcommand prints its own arguments and notes.
//! `docs/commands.md` mirrors this content.

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
  status                                  Print a one-shot system snapshot
  processes                               Print a one-shot process list
  watch                                   Print periodic system snapshots
  info                                    Show build, path and tool information
  doctor                                  Diagnose the local installation
  completions <shell>                     Print bash/zsh/fish/PowerShell completions
  update                                  Update through xpm, pacman, cargo or the binary installer
  addon list                              List installed themes and layouts
  addon available                         List the installable asset catalog
  addon install <name|--all>              Install themes or layouts
  addon download <name|--all>             Alias for addon install
  addon path                              Print theme and layout directories
  mcp                                     Run the MCP server over stdio
  theme list                              List installed themes
  theme available                         List the remote theme catalog
  theme current                           Print the persisted theme
  theme show <theme>                      Show an installed theme
  theme set <theme>                       Persist the active theme
  theme download <theme|--all>            Install themes from the catalog
  theme path                              Print the themes directory
  layout list                             List installed layouts
  layout available                        List the remote layout catalog
  layout current                          Print the persisted layout
  layout show <layout>                    Show an installed layout
  layout set <layout>                     Persist the active layout
  layout download <layout|--all>          Install layouts from the catalog
  layout check <file>                     Validate a layout JSONC/JSON file
  layout path                             Print the layouts directory
  config download <name|--all>            Install a theme or layout from the catalog
  config generate                         Create the base config, theme and layout
  config get <key.path>                   Print a config value
  config set <key.path> <value>           Update and validate a config value
  config list                             Print the effective config as JSON
  config reset --yes                      Restore config.json to defaults
  config path                             Print config and asset directories
  plugin list                             List compiled-in plugins
  plugin install <name|url>               Add a plugin to the kernel manifest
  plugin scaffold <name>                  Create a plugin template
  widget list                             List compiled-in widget packs
  widget install <name|url|path>          Add a widget pack to the kernel manifest
  widget scaffold <name>                  Create a widget pack template

Global options:
  --change-theme <theme>                  Change the active theme and persist it
  --gen-config                            Create config.json, themes/x.jsonc and layouts/dashboard.jsonc
  -V, --version                           Print the version and exit
  -h, --help                              Show this help and exit

Run 'xtop <command> --help' for command-specific help.
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
