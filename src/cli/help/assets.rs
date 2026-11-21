//! Asset, theme and layout help text.

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
  show <theme>         Show an installed theme and its palette
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

pub(crate) const THEME_SHOW: &str = "\
xtop theme show - show an installed theme

Usage:
  xtop theme show <theme>

Prints the source, optional path and metadata, then the normalized theme
model as JSON (background, foreground and the 16-slot palette).

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
  show <layout>        Show an installed layout and its JSONC source
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

pub(crate) const LAYOUT_SHOW: &str = "\
xtop layout show - show an installed layout

Usage:
  xtop layout show <layout>

Prints the source, optional path and metadata, followed by the JSONC source
when the layout is stored in the user layouts directory.

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
  get <key.path>          Print a config value as JSON or text
  set <key.path> <value>  Update and validate a config value
  list                    Print the effective configuration as JSON
  reset --yes             Restore config.json to defaults
  path                    Print config, theme and layout directories

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

pub(crate) const CONFIG_GET: &str = "\
xtop config get - print a configuration value

Usage:
  xtop config get <key.path>

Reads the effective configuration and prints the selected value. Strings are
printed raw; objects and arrays are printed as pretty JSON.

Examples:
  xtop config get theme
  xtop config get alerts.cpu_high
  xtop config get keybindings.quit

Options:
  -h, --help    Show this help and exit";

pub(crate) const CONFIG_SET: &str = "\
xtop config set - update a configuration value

Usage:
  xtop config set <key.path> <value>

The value is parsed as JSON when possible, otherwise as a string. The
resulting configuration is validated against the schema before it is saved.

Examples:
  xtop config set theme tokio
  xtop config set update_interval_ms 500
  xtop config set alerts.cpu_high 80
  xtop config set keybindings.quit '[\"q\",\"ctrl+q\"]'

Options:
  -h, --help    Show this help and exit";

pub(crate) const CONFIG_LIST: &str = "\
xtop config list - print the effective configuration

Usage:
  xtop config list

Prints the merged configuration as pretty JSON.

Options:
  -h, --help    Show this help and exit";

pub(crate) const CONFIG_RESET: &str = "\
xtop config reset - restore config.json to defaults

Usage:
  xtop config reset --yes

Writes the default configuration, including the base theme and primary
layout selection. Requires --yes to avoid accidental data loss.

Options:
  --yes         Confirm the reset
  -h, --help    Show this help and exit";

pub(crate) const ADDON: &str = "\
xtop addon - manage installable themes and layouts

Usage:
  xtop addon <command> [arguments]

Commands:
  list                    List installed themes and layouts
  available               List the official catalog
  install <name|--all>    Install a theme or layout (alias: download)
  download <name|--all>   Install a theme or layout
  path                    Print the theme and layout directories

Options:
  --all                   Install every theme and layout in the catalog
  --force                 Replace existing files
  -h, --help              Show this help and exit

Plugins and widget packs are compile-time developer tooling; use
'xtop plugin' and 'xtop widget' for those.";

pub(crate) const ADDON_LIST: &str = "\
xtop addon list - list installed themes and layouts

Usage:
  xtop addon list

Options:
  -h, --help    Show this help and exit";

pub(crate) const ADDON_AVAILABLE: &str = "\
xtop addon available - list the official catalog

Usage:
  xtop addon available

Fetches the catalog and prints every theme and layout with version, author,
license and installed state.

Options:
  -h, --help    Show this help and exit";

pub(crate) const ADDON_INSTALL: &str = "\
xtop addon install - install themes or layouts from the catalog

Usage:
  xtop addon install <name>
  xtop addon install --all
  xtop addon install <name|--all> --force

Options:
  --all         Install every theme and layout in the catalog
  --force       Replace existing files
  -h, --help    Show this help and exit";

pub(crate) const ADDON_DOWNLOAD: &str = "\
xtop addon download - install themes or layouts from the catalog

Usage:
  xtop addon download <name>
  xtop addon download --all
  xtop addon download <name|--all> --force

This is an alias for 'xtop addon install'.

Options:
  --all         Install every theme and layout in the catalog
  --force       Replace existing files
  -h, --help    Show this help and exit";

pub(crate) const ADDON_PATH: &str = "\
xtop addon path - print the theme and layout directories

Usage:
  xtop addon path

Options:
  -h, --help    Show this help and exit";
