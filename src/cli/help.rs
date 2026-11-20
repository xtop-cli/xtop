//! Help text for the hand-written CLI.
//!
//! The help is layered by command: the global `--help` lists every command
//! and each group/subcommand prints its own arguments and notes.
//! `docs/commands.md` mirrors this content.

pub(crate) const GLOBAL: &str = "\
xtop - cross-platform TUI system monitor and kernel CLI

Usage:
  xtop
  xtop <command> [arguments]
  xtop --change-theme <theme>
  xtop --gen-config
  xtop --help

Commands:
  mcp                                     Run the MCP server over stdio
  theme list                              List installed themes
  theme available                         List the remote theme catalog
  theme current                           Print the persisted theme
  theme set <theme>                       Persist the active theme
  theme download <theme|--all>            Install themes from the catalog
  theme path                              Print the themes directory
  layout list                             List installed layouts
  layout available                        List the remote layout catalog
  layout current                          Print the persisted layout
  layout set <layout>                     Persist the active layout
  layout download <layout|--all>          Install layouts from the catalog
  layout check <file>                     Validate a layout JSONC/JSON file
  layout path                             Print the layouts directory
  config download <name|--all>            Install a theme or layout from the catalog
  config generate                         Create the base config, theme and layout
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
  -h, --help                              Show this help and exit

Run 'xtop <command> --help' for command-specific help.
Without a command, xtop starts the interactive TUI.";

pub(crate) const MCP: &str = "\
xtop mcp - run the MCP server over stdio

Usage:
  xtop mcp

Starts the MCP extension so AI agents can call the samurai plugin tools.
Requires the plugin-samurai and mcp-extension features (enabled by default).

Options:
  -h, --help    Show this help and exit";

pub(crate) const CHANGE_THEME: &str = "\
xtop --change-theme - change the active theme

Usage:
  xtop --change-theme <theme>

Persists the theme in config.json. Running TUI instances pick it up on the
next tick. The deprecated --ct alias is still accepted.

Arguments:
  <theme>       Installed theme name (for example: x, tokio, madrid)

Options:
  -h, --help    Show this help and exit";

pub(crate) const GEN_CONFIG: &str = "\
xtop --gen-config - create the base configuration files

Usage:
  xtop --gen-config

Creates config.json, themes/x.jsonc and layouts/dashboard.jsonc when they are
missing. Existing files are left untouched.

Options:
  -h, --help    Show this help and exit";

pub(crate) const THEME: &str = "\
xtop theme - manage themes

Usage:
  xtop theme <command> [arguments]

Commands:
  list                 List installed themes
  available            List themes in the official catalog
  current              Print the persisted theme
  set <theme>          Persist the active theme
  download <theme|--all> [--force]
                       Install themes from the catalog
  path                 Print the user themes directory

Options:
  -h, --help           Show this help and exit

Run 'xtop theme <command> --help' for command-specific help.";

pub(crate) const THEME_LIST: &str = "\
xtop theme list - list installed themes

Usage:
  xtop theme list

Lists the built-in base theme, downloaded themes and user JSONC files, with
version, author and license metadata when the theme was installed by xtop.

Options:
  -h, --help    Show this help and exit";

pub(crate) const THEME_AVAILABLE: &str = "\
xtop theme available - list the official theme catalog

Usage:
  xtop theme available

Fetches the catalog from the kernel repository (or XTOP_CATALOG_URL) and
prints every theme with its version, author, license and installed state.

Options:
  -h, --help    Show this help and exit";

pub(crate) const THEME_CURRENT: &str = "\
xtop theme current - print the persisted theme

Usage:
  xtop theme current

Options:
  -h, --help    Show this help and exit";

pub(crate) const THEME_SET: &str = "\
xtop theme set - persist the active theme

Usage:
  xtop theme set <theme>

The theme must be installed. Running TUI instances follow the change on the
next tick.

Options:
  -h, --help    Show this help and exit";

pub(crate) const THEME_DOWNLOAD: &str = "\
xtop theme download - install themes from the catalog

Usage:
  xtop theme download <theme>
  xtop theme download --all
  xtop theme download <theme|--all> --force

Arguments:
  <theme>       Theme id or display name from the catalog

Options:
  --all         Download every theme in the catalog
  --force       Replace an existing file
  -h, --help    Show this help and exit";

pub(crate) const THEME_PATH: &str = "\
xtop theme path - print the user themes directory

Usage:
  xtop theme path

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT: &str = "\
xtop layout - manage layouts

Usage:
  xtop layout <command> [arguments]

Commands:
  list                 List the effective installed layouts
  available            List layouts in the official catalog
  current              Print the persisted layout
  set <layout>         Persist the active layout
  download <layout|--all> [--force]
                       Install layouts from the catalog
  install <layout|--all> [--force]
                       Alias for download (kept for compatibility)
  check <file>         Validate a layout JSONC/JSON file
  path                 Print the user layouts directory

Options:
  -h, --help           Show this help and exit

Run 'xtop layout <command> --help' for command-specific help.";

pub(crate) const LAYOUT_LIST: &str = "\
xtop layout list - list the effective installed layouts

Usage:
  xtop layout list

Lists the embedded base Dashboard plus downloaded and user layouts. The
current layout is marked with an asterisk.

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_AVAILABLE: &str = "\
xtop layout available - list the official layout catalog

Usage:
  xtop layout available

Fetches the catalog from the kernel repository (or XTOP_CATALOG_URL) and
prints every layout with its category, version, author, license and
installed state.

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_CURRENT: &str = "\
xtop layout current - print the persisted layout

Usage:
  xtop layout current

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_SET: &str = "\
xtop layout set - persist the active layout

Usage:
  xtop layout set <layout>

The layout must be installed. Running TUI instances follow the change on the
next tick.

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_DOWNLOAD: &str = "\
xtop layout download - install layouts from the catalog

Usage:
  xtop layout download <layout>
  xtop layout download --all
  xtop layout download <layout|--all> --force

Arguments:
  <layout>      Layout id or display name from the catalog

Options:
  --all         Download every layout in the catalog
  --force       Replace an existing file
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_INSTALL: &str = "\
xtop layout install - install layouts from the catalog

Usage:
  xtop layout install <layout>
  xtop layout install --all
  xtop layout install <layout|--all> --force

This is an alias for 'xtop layout download'. It installs layouts from the
official catalog, not only from the community folder.

Options:
  --all         Download every layout in the catalog
  --force       Replace an existing file
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_CHECK: &str = "\
xtop layout check - validate a layout file

Usage:
  xtop layout check <file.jsonc|file.json>

Runs the same parser used at runtime and reports structural validity. It
does not check whether widget ids resolve in the current kernel build.

Options:
  -h, --help    Show this help and exit";

pub(crate) const LAYOUT_PATH: &str = "\
xtop layout path - print the user layouts directory

Usage:
  xtop layout path

Options:
  -h, --help    Show this help and exit";

pub(crate) const CONFIG: &str = "\
xtop config - manage external assets and config paths

Usage:
  xtop config <command> [arguments]

Commands:
  download <name|--all> [--force]
                          Download themes or layouts from the catalog
  generate                Create config.json, the base theme and the base layout
  path                     Print config, theme and layout directories

Options:
  -h, --help              Show this help and exit

Run 'xtop config <command> --help' for command-specific help.";

pub(crate) const CONFIG_DOWNLOAD: &str = "\
xtop config download - install themes or layouts from the catalog

Usage:
  xtop config download <name>
  xtop config download --all
  xtop config download <name|--all> --force

Arguments:
  <name>        Theme or layout id/display name from the catalog

Options:
  --all         Download every theme and layout in the catalog
  --force       Replace existing files
  -h, --help    Show this help and exit";

pub(crate) const CONFIG_GENERATE: &str = "\
xtop config generate - create the base configuration files

Usage:
  xtop config generate

Creates config.json, themes/x.jsonc and layouts/dashboard.jsonc when they are
missing. Existing files are left untouched.

Options:
  -h, --help    Show this help and exit";

pub(crate) const CONFIG_PATH: &str = "\
xtop config path - print configuration and asset paths

Usage:
  xtop config path

Prints the config directory, themes directory, layouts directory and the
installed-assets manifest path.

Options:
  -h, --help    Show this help and exit";

pub(crate) const PLUGIN: &str = "\
xtop plugin - manage plugins compiled into the kernel

Usage:
  xtop plugin <command> [arguments]

Commands:
  list                 List plugins wired into the kernel manifest
  install <name|url>   Add a plugin as an optional git dependency and feature
  scaffold <name>      Create a plugin crate under plugins-dev/

Options:
  -h, --help           Show this help and exit

install and scaffold edit the kernel source tree and require a source
checkout plus cargo. Run 'xtop plugin <command> --help' for details.";

pub(crate) const PLUGIN_LIST: &str = "\
xtop plugin list - list plugins wired into the kernel

Usage:
  xtop plugin list

Reads the kernel Cargo.toml and prints dependencies with a matching
dep:<name> feature.

Options:
  -h, --help    Show this help and exit";

pub(crate) const PLUGIN_INSTALL: &str = "\
xtop plugin install - add a plugin to the kernel manifest

Usage:
  xtop plugin install <name|url>

Arguments:
  <name>    Plugin name in github.com/xtop-cli/plugins (xtop-plugin-<name>)
  <url>     Git URL containing an xtop-plugin-<name> crate at the root,
            under plugins/, or under crates/

Adds an optional git dependency and feature flag to the kernel Cargo.toml,
then runs cargo check. The plugin is not enabled by default. Requires git
and cargo.

Options:
  -h, --help    Show this help and exit";

pub(crate) const PLUGIN_SCAFFOLD: &str = "\
xtop plugin scaffold - create a plugin crate template

Usage:
  xtop plugin scaffold <name>

Creates plugins-dev/xtop-plugin-<name> with Cargo.toml and src/lib.rs.
Requires a source checkout.

Options:
  -h, --help    Show this help and exit";

pub(crate) const WIDGET: &str = "\
xtop widget - manage widget packs compiled into the kernel

Usage:
  xtop widget <command> [arguments]

Commands:
  list                        List widget packs wired into the kernel
  install <name|url|path>     Add a widget pack as an optional dependency
  scaffold <name>             Create a single-widget pack under widgets-dev/

Options:
  -h, --help                  Show this help and exit

install and scaffold edit the kernel source tree and require a source
checkout plus cargo. Run 'xtop widget <command> --help' for details.";

pub(crate) const WIDGET_LIST: &str = "\
xtop widget list - list widget packs wired into the kernel

Usage:
  xtop widget list

Reads the kernel Cargo.toml and the compile-time pack catalog, then prints
the packs enabled in this build.

Options:
  -h, --help    Show this help and exit";

pub(crate) const WIDGET_INSTALL: &str = "\
xtop widget install - add a widget pack to the kernel

Usage:
  xtop widget install <name|url|path>

Arguments:
  <name>    Pack name in github.com/xtop-cli/widgets (xtop-widget-<name>)
  <url>     Git URL containing an xtop-widget-<name> crate
  <path>    Local directory containing the pack crate

Adds an optional dependency and feature flag, appends a row and registry arm
to src/tui/layout/pack_table.rs, then runs cargo check. The pack is not
enabled by default. Requires cargo.

Options:
  -h, --help    Show this help and exit";

pub(crate) const WIDGET_SCAFFOLD: &str = "\
xtop widget scaffold - create a single-widget pack template

Usage:
  xtop widget scaffold <name>

Creates widgets-dev/xtop-widget-<name> with a Cargo.toml, a render function
and the registration boilerplate. Requires a source checkout.

Options:
  -h, --help    Show this help and exit";

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
