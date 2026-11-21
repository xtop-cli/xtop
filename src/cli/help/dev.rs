//! Developer tooling help text for plugins and widget packs.

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
