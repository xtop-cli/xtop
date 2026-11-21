# Command reference

This document lists every command, argument and option exposed by the
`xtop` binary. The interactive TUI is the default mode; the management
commands configure themes, layouts, plugins and widget packs.

Help is layered:

- `xtop --help` (or `xtop -h`) lists the top-level commands and groups.
- `xtop <group> --help` lists the commands in one group.
- `xtop <group> <command> --help` lists the arguments and notes for one
  command; `xtop status <section> --help` documents one status section.
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

Running `xtop` with no arguments starts the TUI. An unrecognized
positional command also starts the TUI; this preserves the original entry
behavior. Unknown flags are usage errors (exit status 2) and never open the
TUI. There are no per-metric top-level flags such as `--cpu`, `--ram` or
`--disk`; use `xtop status`, `xtop processes` and `xtop watch` for metric
state.

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
| `xtop status [section] [--json]` | Print a snapshot or one section; see [`xtop status`](#xtop-status). |
| `xtop processes [options]` | Print a sortable, filterable process list. |
| `xtop watch [options]` | Print periodic system snapshots. |
| `xtop info [--json]` | Show build, path and tool information. |
| `xtop doctor [--json]` | Diagnose the local installation. |
| `xtop completions <shell>` | Print bash, zsh, fish or PowerShell completions. |
| `xtop update [options]` | Update through the detected installation medium. |
| `xtop theme ...` | Manage themes; run `xtop theme --help`. |
| `xtop layout ...` | Manage layouts; run `xtop layout --help`. |
| `xtop addon ...` | Manage installable themes and layouts; run `xtop addon --help`. |
| `xtop config ...` | Manage configuration and paths; run `xtop config --help`. |
| `xtop plugin ...` | Manage compiled plugins; run `xtop plugin --help`. |
| `xtop widget ...` | Manage widget packs; run `xtop widget --help`. |
| `xtop mcp` | Start the MCP server over stdio for AI agents. |

The table lists the top-level commands only; each group prints its own
leaves with `xtop <group> --help`. Group aliases: `xtop themes` is
`xtop theme`, `xtop layouts` is `xtop layout`, and `xtop addons` is
`xtop addon`.

## TUI## TUI

```
xtop
```

Starts the interactive system monitor. Keybindings, layouts, fullscreen
mode, the command bar and process filtering are documented in
[usage.md](usage.md). The TUI requires stdin and stdout to be TTYs; a
redirected invocation exits with a clear error instead of emitting terminal
escape sequences.

## Headless monitoring

These commands do not start the TUI and are intended for scripts, terminals
without a full-screen session and quick checks.

### `xtop status`

```
xtop status [section] [--json]
```

Without a section, prints the complete snapshot: CPU, memory (RAM), swap,
load, uptime, disks, disk I/O, network interfaces, batteries and GPUs. The sampler
takes two samples with a short pause so CPU and rate values are meaningful
in the first output.

Each section is a quick query for one part of the state:

| Section | Reports | JSON key |
| --- | --- | --- |
| `summary` | Complete snapshot (default) | all keys |
| `host` | Hostname, OS, kernel, desktop, shell and CPU model | `host` |
| `cpu` | Aggregate usage/temperature and per-core details | `cpu` |
| `memory` | RAM total, used, available, free and percentage | `memory` |
| `swap` | Swap total, used, free and percentage | `swap` |
| `load` | One, five and fifteen minute load averages | `load` |
| `uptime` | System uptime | `uptime_seconds` |
| `disks` | Mount points, filesystems and space | `disks` |
| `disk-io` | Per-device read/write totals and rates | `disk_io` |
| `network` | Interfaces with current RX/TX rates | `networks` |
| `battery` | Batteries with charge percentage and state | `batteries` |
| `gpu` | GPUs with usage, temperature and memory | `gpus` |

Aliases: `mem`/`ram` for `memory`, `disk_io`/`io` for `disk-io`, `net` for
`network`, `batteries` for `battery`, `gpus` for `gpu`, and `all` for
`summary`.

`--json` prints one machine-readable object with schema version 1. With a
section, the object keeps `schema`, `kind` and only that section's key; for
example `xtop status memory --json` returns `memory`.

For the process list use `xtop processes`; it has its own sorting, filtering
and JSON options.

### `xtop processes`

```
xtop processes [--json] [--sort cpu|memory|mem|pid|name] [--asc|--desc]
               [--limit N] [--filter TEXT]
```

Prints a process table. CPU and memory are sampled over a short interval.
The default sort is CPU descending and the default limit is 20; `--limit 0`
prints every process in the snapshot. `--filter` matches process names.
`mem` is accepted as an alias for `memory`. `--json` omits the process
environment and full argv.

### `xtop watch`

```
xtop watch [--json|--jsonl] [--interval MS] [--count N]
```

Repeats the status snapshot. The default interval is 1000 ms; values below
100 ms are clamped. Without `--count`, the command runs until it is
interrupted. `--jsonl` (and `--json`) prints one compact JSON object per
sample.

## Current state inspection

`xtop status` is the one-shot view of the current host state; the
[status section table](#xtop-status) above is the metric map. Use the
section commands for quick answers: `xtop status memory`, `xtop status cpu`,
`xtop status disk-io`, and so on. There is no separate top-level `--ram`
flag.

The rest of the state is available from these commands:

| Command | Current state reported |
| --- | --- |
| `xtop processes [--json]` | Process table; JSON includes `pid`, `name`, `cpu_percent`, `memory_bytes`, `user_id`, `state`, `cmd`, `exe_path`, `parent_pid`, `start_time_epoch`, `run_time_seconds`, `thread_count`, `open_files` and `session_id`. |
| `xtop watch [--jsonl]` | Repeated snapshots for live monitoring or streaming into another tool. |
| `xtop info [--json]` | Build, enabled features, config/asset paths, local asset counts and tool availability. |
| `xtop doctor [--json]` | Installation diagnostics: directory writability, config/manifest parsing, base assets, catalog and external tools. |
| `xtop config list` | Effective persisted configuration. |
| `xtop config path` | Config, themes, layouts and installed-assets paths. |
| `xtop theme current` / `xtop layout current` | Active theme and layout. |
| `xtop addon list` / `xtop plugin list` / `xtop widget list` | Installed assets, compiled plugins and widget packs. |

For scripting, combine the section output with your JSON tool of choice:

```sh
xtop status memory
xtop status memory --json
xtop status cpu --json
xtop processes --json --sort memory --limit 10
xtop watch --jsonl --count 1
```

See [core.md](core.md) for the snapshot model, providers and platform
coverage behind these commands.

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
xtop completions <bash|zsh|fish|powershell|pwsh>
```

Prints a completion script to stdout. `pwsh` is an alias for `powershell`.
Source the script from the shell startup file or install it in the shell's
completion directory.

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
`https://raw.githubusercontent.com/xtop-cli/xtop/main/catalog.json`. Set
`XTOP_CATALOG_URL` to use a local file, mirror or test catalog.

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
| `--method <auto\|xpm\|pacman\|cargo\|binary\|bin>` | Override detection. `bin` is an alias for `binary`. |
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

The plural group names `xtop themes`, `xtop layouts` and `xtop addons` are
aliases for `xtop theme`, `xtop layout` and `xtop addon`.

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
xtop status memory
xtop status cpu --json
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
