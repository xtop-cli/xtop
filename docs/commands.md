# Command reference

This document lists every command, argument and option exposed by the
`xtop` binary. The interactive TUI is the default mode; the management
commands configure themes, layouts, plugins and widget packs.

Help is layered:

- `xtop --help` (or `xtop -h`) lists every command and global option.
- `xtop <group> --help` lists the commands in one group.
- `xtop <group> <command> --help` lists the arguments and notes for one
  command.
- `xtop --change-theme --help` and `xtop --gen-config --help` document the
  global flags.

Help output is written to stdout and exits with status 0. Usage errors are
written to stderr and exit with status 2. Runtime failures exit with status
1.

## Global usage

```
xtop
xtop <command> [arguments]
xtop --change-theme <theme>
xtop --gen-config
xtop --help
```

Running `xtop` with no arguments starts the TUI. An unrecognized first
argument also starts the TUI; this preserves the original entry behavior.

### Global options

| Option | Description |
| --- | --- |
| `--change-theme <theme>` | Change the active theme and persist it to `config.json`. Running instances pick up the change on the next tick. `--ct` remains accepted as a deprecated alias. |
| `--gen-config` | Create `config.json`, `themes/x.jsonc` and `layouts/dashboard.jsonc` when missing. |
| `-h`, `--help` | Print global help and exit. |

## Commands

| Command | Description |
| --- | --- |
| `xtop` | Start the interactive TUI system monitor. |
| `xtop mcp` | Start the MCP server over stdio for AI agents. |
| `xtop theme list` | List installed themes. |
| `xtop theme available` | List the official theme catalog. |
| `xtop theme current` | Print the persisted theme. |
| `xtop theme set <theme>` | Persist the active theme. |
| `xtop theme download <theme>` | Install one theme, or all themes with `--all`, from the catalog. |
| `xtop theme path` | Print the user themes directory. |
| `xtop layout list` | List effective installed layouts. |
| `xtop layout available` | List the official layout catalog. |
| `xtop layout current` | Print the persisted layout. |
| `xtop layout set <layout>` | Persist the active layout. |
| `xtop layout download <layout>` | Install one layout, or all layouts with `--all`, from the catalog. |
| `xtop layout install <layout>` | Alias for `layout download`; accepts the same `--all` flag. |
| `xtop layout check <file>` | Validate a layout JSONC/JSON file. |
| `xtop layout path` | Print the user layouts directory. |
| `xtop config download <name>` | Download a theme or layout by name, or everything with `--all`. |
| `xtop config generate` | Create the base config and assets. |
| `xtop config path` | Print config and asset directories. |
| `xtop plugin list` | List plugins wired into the kernel manifest. |
| `xtop plugin install <name\|url>` | Add a plugin as an optional git dependency and feature. |
| `xtop plugin scaffold <name>` | Create a plugin crate template under `plugins-dev/`. |
| `xtop widget list` | List widget packs wired into the kernel. |
| `xtop widget install <name\|url\|path>` | Add a widget pack as an optional dependency and feature. |
| `xtop widget scaffold <name>` | Create a single-widget pack template under `widgets-dev/`. |

## TUI

```
xtop
```

Starts the interactive system monitor. Keybindings, layouts, fullscreen
mode, process search and the command palette are documented in
[usage.md](usage.md).

## MCP server

```
xtop mcp
```

Starts the MCP extension over stdio so AI agents can call the samurai
plugin tools. The command requires the `plugin-samurai` and `mcp-extension`
features, which are enabled in the default build.

`xtop mcp` takes no arguments. `xtop mcp --help` prints the command help.

## Themes

```
xtop theme <command> [arguments]
```

Themes are JSONC files with an explicit background/foreground pair and a
16-slot palette. Only the base `x` theme is embedded in the binary. Every
other theme is external and is installed from the official catalog into
`$XDG_CONFIG_HOME/xtop/themes` (Linux), the Application Support directory
(macOS) or AppData (Windows).

### `xtop theme list`

```
xtop theme list
```

Lists the base theme, downloaded themes and user JSONC files. Downloaded
themes show their version, author and license from the local
`assets.json` manifest.

### `xtop theme available`

```
xtop theme available
```

Fetches the official catalog and lists every theme with its id, display
name, version, author, license and installed state. The catalog source can
be overridden with `XTOP_CATALOG_URL`.

### `xtop theme current`

```
xtop theme current
```

Prints the theme persisted in `config.json`.

### `xtop theme set`

```
xtop theme set <theme>
```

Persists the selected theme. The theme must be installed; `xtop theme
download <theme>` installs it first. Running TUI instances follow the
change on the next tick.

### `xtop theme download`

```
xtop theme download <theme>
xtop theme download --all
xtop theme download <theme|--all> --force
```

Downloads one theme or every theme from the official catalog. Files are
verified against the catalog SHA-256, validated as themes, and written
atomically. Existing files are skipped unless `--force` is given.

### `xtop theme path`

```
xtop theme path
```

Prints the user themes directory.

## Layouts

```
xtop layout <command> [arguments]
```

Layouts are JSONC trees of splits and widget leaves. The primary
`Dashboard` layout is embedded in the binary; every other layout is an
external catalog asset or a user file in the user layouts directory.

### `xtop layout list`

```
xtop layout list
```

Lists the embedded Dashboard, downloaded layouts and user layouts. The
current layout is marked with an asterisk.

### `xtop layout available`

```
xtop layout available
```

Fetches the official catalog and lists every layout with its id, display
name, category, version, author, license and installed state.

### `xtop layout current`

```
xtop layout current
```

Prints the layout persisted in `config.json`, or the current layout mode
when no name is set.

### `xtop layout set`

```
xtop layout set <layout>
```

Persists the selected layout. The layout must be installed. Running TUI
instances follow the change on the next tick and reload the layouts
directory if the layout was downloaded after they started.

### `xtop layout download`

```
xtop layout download <layout>
xtop layout download --all
xtop layout download <layout|--all> --force
```

Downloads one layout or every layout from the official catalog. Files are
verified against the catalog SHA-256, validated with the runtime parser,
and written atomically. Existing files are skipped unless `--force` is
given.

`xtop layout install` is an alias for `xtop layout download`.

### `xtop layout check`

```
xtop layout check <file.jsonc|file.json>
```

Runs the same parser used at runtime and reports structural validity. It
does not check whether widget ids resolve in the current kernel build. A
valid file exits with status 0; an invalid file exits with status 1.

### `xtop layout path`

```
xtop layout path
```

Prints the user layouts directory.

## Config and assets

```
xtop config <command> [arguments]
```

### `xtop config download`

```
xtop config download <name>
xtop config download --all
xtop config download <name|--all> --force
```

Downloads a theme or layout by catalog id or display name. `--all` installs
every theme and layout in the catalog. If the name matches one theme and
one layout, both are installed.

### `xtop config generate`

```
xtop config generate
```

Creates `config.json`, `themes/x.jsonc` and `layouts/dashboard.jsonc` when
missing. This is the same operation as `xtop --gen-config`.

### `xtop config path`

```
xtop config path
```

Prints the config directory, themes directory, layouts directory and
installed-assets manifest path.

### Catalog and installed metadata

The official catalog is `catalog.json` at the root of the
`github.com/xtop-cli/xtop` repository. Each entry carries:

- `id` and display `name`;
- `version`, `author` and `license`;
- `file` (safe destination file name), `url` (raw asset URL) and `sha256`.

The default catalog URL is
`https://raw.githubusercontent.com/xtop-cli/xtop/main/catalog.json`; set
`XTOP_CATALOG_URL` to use a local file, mirror or test catalog.

Downloads are recorded in `$XDG_CONFIG_HOME/xtop/assets.json` (or the
platform equivalent) with version, author, license and checksum metadata.
The catalog is the source of truth; the manifest describes what is
installed locally.

## Plugins

```
xtop plugin <command> [arguments]
```

Plugins are compiled into the kernel through Cargo git dependencies and
feature flags. `install` and `scaffold` edit the kernel source tree, so they
require a checkout of the kernel repository and a working `cargo`.

### `xtop plugin list`

```
xtop plugin list
```

Reads the kernel `Cargo.toml` and prints plugin dependencies that have a
matching `dep:<name>` feature entry. Contract crates such as
`xtop-plugin-api` are excluded.

### `xtop plugin install`

```
xtop plugin install <name|url>
```

Arguments:

- `<name>`: plugin name from `github.com/xtop-cli/plugins`. The crate is
  `xtop-plugin-<name>`.
- `<url>`: git URL containing an `xtop-plugin-<name>` crate at the
  repository root, under `plugins/`, or under `crates/`.

The command clones the source shallowly, locates the crate, adds an
optional git dependency and a feature flag to the kernel `Cargo.toml`, and
runs `cargo check`. The plugin is registered but not enabled by default.
Add the feature to the default list or build with
`--features <feature>`, then add the registration line in
`src/core/bootstrap.rs` under the feature guard.

Requires `git` and `cargo`.

### `xtop plugin scaffold`

```
xtop plugin scaffold <name>
```

Creates `plugins-dev/xtop-plugin-<name>/` with a `Cargo.toml` and
`src/lib.rs` template that implements the plugin API. Requires a kernel
source checkout.

## Widget packs

```
xtop widget <command> [arguments]
```

Widget packs are compile-time crates registered in the kernel manifest and
in the pack catalog at `src/tui/layout/pack_table.rs`. `install` and
`scaffold` edit the kernel sources and require a checkout plus `cargo`.

### `xtop widget list`

```
xtop widget list
```

Reads the kernel `Cargo.toml` and the compile-time pack catalog, then
prints the packs enabled in this build. The base pack is always listed;
optional packs show their feature name.

### `xtop widget install`

```
xtop widget install <name|url|path>
```

Arguments:

- `<name>`: pack name from `github.com/xtop-cli/widgets`. The crate is
  `xtop-widget-<name>`.
- `<url>`: git URL containing an `xtop-widget-<name>` crate.
- `<path>`: local directory containing the pack crate.

The command adds an optional dependency and a `widget-<name>` feature,
appends a row and a registry arm to `src/tui/layout/pack_table.rs`, and runs
`cargo check`. The pack is not enabled by default. Enable the feature and
select the pack through `style.pack` or `style.widgets.<name>.pack`.

Requires `cargo`. Git URLs also require `git`.

### `xtop widget scaffold`

```
xtop widget scaffold <name>
```

Creates `widgets-dev/xtop-widget-<name>/` with a `Cargo.toml`, a render
function and the registration boilerplate. Requires a kernel source
checkout.

## Environment variables

| Variable | Effect |
| --- | --- |
| `XDG_CONFIG_HOME` | Overrides the base Linux configuration directory. The kernel stores its files under `$XDG_CONFIG_HOME/xtop`. |
| `XTOP_CATALOG_URL` | Overrides the catalog source used by `theme available`, `layout available` and the download commands. Accepts an HTTP(S) URL, a local path or a `file://` URL. |
| `XTOP_MAX_PROCESSES` | Overrides the process snapshot cap. Default: 200. |
| `XTOP_WASM_DIR` | Overrides the runtime WASM widget directory. |
| `XTOP_EXTERNAL_DIR` | Overrides the external widget descriptor directory. |

macOS and Windows use their platform configuration directories; see
[configuration.md](configuration.md).

## Exit codes

| Status | Meaning |
| --- | --- |
| 0 | Success, help output, or a clean MCP shutdown. |
| 1 | Runtime failure, such as an unknown theme, a failed download, an invalid layout, a failed clone or a failed `cargo check`. |
| 2 | Usage error, such as a missing argument or an unknown subcommand. |

## Examples

```sh
# Start the TUI
xtop

# Print global help and scoped help
xtop --help
xtop theme --help
xtop theme download --help

# Create the base configuration without starting the TUI
xtop --gen-config

# Inspect what is installed and what is available
xtop theme list
xtop theme available
xtop layout list
xtop layout available

# Install and activate a theme or layout
xtop theme download tokio
xtop theme set tokio
xtop layout download "CPU Focus"
xtop layout set "CPU Focus"

# Install everything from the official catalog
xtop config download --all

# Change the theme of a running instance
xtop --change-theme madrid
```

## Related documentation

- [usage.md](usage.md): TUI keybindings and overlays.
- [configuration.md](configuration.md): configuration schema and paths.
- [customization.md](customization.md): themes, layouts and runtime widgets.
- [plugin.md](plugin.md): plugin architecture and authoring.
- [core.md](core.md): what the reusable kernel core provides.
