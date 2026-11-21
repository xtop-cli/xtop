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
xtop --version
xtop --help
```

Running `xtop` with no arguments starts the TUI. An unrecognized first
argument also starts the TUI; this preserves the original entry behavior.

### Global options

| Option | Description |
| --- | --- |
| `--change-theme <theme>` | Change the active theme and persist it to `config.json`. Running instances pick up the change on the next tick. `--ct` remains accepted as a deprecated alias. |
| `--gen-config` | Create `config.json`, `themes/x.jsonc` and `layouts/dashboard.jsonc` when missing. |
| `-V`, `--version` | Print the package version and exit. |
| `-h`, `--help` | Print global help and exit. |

## Commands

| Command | Description |
| --- | --- |
| `xtop` | Start the interactive TUI system monitor. |
| `xtop status` | Print a one-shot system snapshot. |
| `xtop processes` | Print a one-shot process list. |
| `xtop watch` | Print periodic system snapshots. |
| `xtop info` | Show build, path and tool information. |
| `xtop doctor` | Diagnose the local installation. |
| `xtop completions <shell>` | Print bash, zsh, fish or PowerShell completions. |
| `xtop update` | Update through the detected installation medium. |
| `xtop mcp` | Start the MCP server over stdio for AI agents. |
| `xtop theme list` | List installed themes. |
| `xtop theme available` | List the official theme catalog. |
| `xtop theme current` | Print the persisted theme. |
| `xtop theme show <theme>` | Show an installed theme and its palette. |
| `xtop theme set <theme>` | Persist the active theme. |
| `xtop theme download <theme>` | Install one theme, or all themes with `--all`, from the catalog. |
| `xtop theme path` | Print the user themes directory. |
| `xtop layout list` | List effective installed layouts. |
| `xtop layout available` | List the official layout catalog. |
| `xtop layout current` | Print the persisted layout. |
| `xtop layout show <layout>` | Show an installed layout and its JSONC source. |
| `xtop layout set <layout>` | Persist the active layout. |
| `xtop layout download <layout>` | Install one layout, or all layouts with `--all`, from the catalog. |
| `xtop layout install <layout>` | Alias for `layout download`; accepts the same `--all` flag. |
| `xtop layout check <file>` | Validate a layout JSONC/JSON file. |
| `xtop layout path` | Print the user layouts directory. |
| `xtop config download <name>` | Download a theme or layout by name, or everything with `--all`. |
| `xtop config generate` | Create the base config and assets. |
| `xtop config get <key.path>` | Print one configuration value. |
| `xtop config set <key.path> <value>` | Update and validate a configuration value. |
| `xtop config list` | Print the effective configuration as JSON. |
| `xtop config reset --yes` | Restore `config.json` to defaults. |
| `xtop config path` | Print config and asset directories. |
| `xtop addon list` | List installed themes and layouts. |
| `xtop addon available` | List the installable asset catalog. |
| `xtop addon install <name>` | Install a theme or layout, or everything with `--all`. |
| `xtop addon download <name>` | Alias for `addon install`. |
| `xtop addon path` | Print theme and layout directories. |
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

## Headless monitoring

These commands do not start the TUI and are intended for scripts, terminals
without a full-screen session and quick checks.

### `xtop status`

```
xtop status [--json]
```

Samples CPU, memory, swap, load, uptime, disks, network interfaces,
batteries and GPUs. The sampler takes two samples with a short pause so CPU
and rate values are meaningful in the first output.

`--json` prints one machine-readable object with schema version 1.

### `xtop processes`

```
xtop processes [--json] [--sort cpu|memory|pid|name] [--asc|--desc]
               [--limit N] [--filter TEXT]
```

Prints a process table. CPU and memory are sampled over a short interval.
The default sort is CPU descending and the default limit is 20; `--limit 0`
prints every process in the snapshot. `--filter` matches process names.
`--json` omits the process environment and full argv.

### `xtop watch`

```
xtop watch [--json|--jsonl] [--interval MS] [--count N]
```

Repeats the status snapshot. The default interval is 1000 ms; values below
100 ms are clamped. Without `--count`, the command runs until it is
interrupted. `--jsonl` (and `--json`) prints one compact JSON object per
sample.

## Diagnostics

### `xtop info`

```
xtop info [--json]
```

Prints the version, target, enabled features, config/asset paths, local
asset-file counts and whether `curl`, `git` and `cargo` are available.

### `xtop doctor`

```
xtop doctor [--json]
```

Checks config-directory writability, `config.json` and `assets.json`
parsing, the base theme/layout files, the catalog URL and external tools.
Warnings do not fail; failed checks exit with status 1.

## Shell completions

```
xtop completions <bash|zsh|fish|powershell>
```

Prints a completion script to stdout. Source it from the shell startup file
or install it in the shell's completion directory.

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

### `xtop theme show`

```
xtop theme show <theme>
```

Prints source, optional path and version/author/license metadata, followed by
the normalized theme model as JSON.

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

### `xtop layout show`

```
xtop layout show <layout>
```

Prints source, optional path and metadata, followed by the JSONC source when
the layout is stored in the user layouts directory.

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

### `xtop config get`

```
xtop config get <key.path>
```

Prints the selected effective configuration value. Strings are printed raw;
objects and arrays are printed as pretty JSON. Paths use dots and array
indices, for example `alerts.cpu_high` or `keybindings.quit.0`.

### `xtop config set`

```
xtop config set <key.path> <value>
```

Parses the value as JSON when possible, otherwise as a string, then validates
the whole configuration before saving. Theme and layout names are checked
against the installed set. Examples:

```sh
xtop config set theme x
xtop config set update_interval_ms 500
xtop config set alerts.cpu_high 80
xtop config set keybindings.quit '["q","ctrl+q"]'
```

### `xtop config list`

```
xtop config list
```

Prints the effective configuration as pretty JSON.

### `xtop config reset`

```
xtop config reset --yes
```

Restores the default configuration. `--yes` is required to avoid an
accidental reset.

### Catalog and installed metadata

The official catalog is `catalog.json` at the root of the
`github.com/xtop-cli/xtop` repository. Each entry carries:

- `id` and display `name`;
- `version`, `author` and `license`;
- `file` (safe destination file name), `url` (raw asset URL) and `sha256`.

The default catalog URL is
`https://raw.githubusercontent.com/xtop-cli/xtop/main/catalog.json`. Release
branches fall back to their release catalog while the file is not yet on
`main`. Set `XTOP_CATALOG_URL` to use a local file, mirror or test catalog.

Downloads are recorded in `$XDG_CONFIG_HOME/xtop/assets.json` (or the
platform equivalent) with version, author, license and checksum metadata.
The catalog is the source of truth; the manifest describes what is
installed locally.

## Updates

```
xtop update [options]
```

Detects how the running binary was installed and uses the same medium:

- **xpm**: package owner from `xpm query --owner`, updated with
  `xpm install --needed <pkg>`.
- **pacman**: package owner from `pacman -Qoq`, updated with
  `pacman -S --needed <pkg>`.
- **cargo**: detected from the cargo bin directory or `cargo install --list`,
  updated with `cargo install --git https://github.com/xtop-cli/xtop --locked --force`.
- **binary**: standalone release binary, updated with the official prebuilt
  installer for the current platform and binary directory.

Options:

| Option | Description |
| --- | --- |
| `--check` | Print installed/latest versions and whether an update exists. |
| `--dry-run` | Detect the medium and print the command without running it. |
| `--method <auto\|xpm\|pacman\|cargo\|binary>` | Override detection. |
| `--yes`, `-y` | Non-interactive package-manager confirmation and `sudo -n`. |
| `--force` | Reinstall or force a cargo update. |
| `--all-features` | Cargo only: rebuild with every feature. |
| `--version <version>` | Cargo/binary only: target a specific release. |
| `--json` | Print a machine-readable result. |

Package-manager updates use `sudo` when the process is not running as root.
The command never writes configuration outside the platform config directory
(`~/.config/xtop` on Linux, or the macOS/Windows equivalent).

## Addons

```
xtop addon <command> [arguments]
```

Addons are the installable, non-core assets: themes and layouts. They are
plain JSONC files in the user config directory and can be installed one by
one or together:

```sh
xtop addon list
xtop addon available
xtop addon install <name>
xtop addon install --all
xtop addon install <name|--all> --force
xtop addon path
```

`xtop addon install` and `xtop addon download` are interchangeable. Each
entry carries its version, author and license, and downloads are verified
against the catalog SHA-256.

The plural group names `xtop themes` and `xtop layouts` are aliases for
`xtop theme` and `xtop layout`.

Plugins and widget packs are not runtime addons: `xtop plugin` and
`xtop widget` edit the kernel source and rebuild it. They are documented
below.

Process-tree example: once the releases are merged, download the opt-in
layout with `xtop layout download process_tree` (or `xtop addon install
process_tree`); it is not part of the embedded default set.

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

# Print version, global help and scoped help
xtop --version
xtop update --check
xtop update --dry-run
xtop --help
xtop theme --help
xtop theme download --help

# Headless monitoring
xtop status
xtop status --json
xtop processes --sort memory --limit 10
xtop watch --count 5 --interval 2000
xtop info
xtop doctor

# Shell completions
xtop completions bash

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

# Configuration management
xtop config get theme
xtop config set update_interval_ms 500
xtop config list
xtop config reset --yes

# Change the theme of a running instance
xtop --change-theme madrid
```

## Related documentation

- [usage.md](usage.md): TUI keybindings and overlays.
- [configuration.md](configuration.md): configuration schema and paths.
- [customization.md](customization.md): themes, layouts and runtime widgets.
- [plugin.md](plugin.md): plugin architecture and authoring.
- [core.md](core.md): what the reusable kernel core provides.
