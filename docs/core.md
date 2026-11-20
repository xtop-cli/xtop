# Kernel core

The kernel is one Cargo package with one library (`src/lib.rs`) and one
binary (`src/main.rs`). The library is split into three areas with a strict
dependency direction:

```
core  <-  tui
  ^
  |
 cli
```

`src/core` is the reusable part. It does not render terminal frames and does
not parse command-line arguments; the TUI and the CLI consume it.

## Package layout

```
src/
  lib.rs                 library surface: core, cli and tui
  main.rs                binary entry: xtop::cli::run()
  core/
    assets.rs            default asset seeding and config persistence
    bootstrap.rs         provider, theme, layout and plugin assembly
    config/              persisted schema, keybindings and platform paths
    plugins/             plugin manager, host views and extension host
    providers/           system sampling and platform probes
    state/               live state, histories, process view and sorting
    theme/               theme model, loader, persistence and contrast
  cli/                   argument dispatch and management commands
  tui/                   run loop, input, rendering and terminal backends
```

The audit enforces the direction: `core` never references `cli` or `tui`;
`tui` never references `cli`; the CLI reaches the TUI only through
`src/cli/commands/tui.rs`.

## System data providers

`core/providers` produces a `SystemSnapshot` through the
`SystemDataProvider` contract from `xtop-plugin-api`.

- `SysinfoProvider` is the primary provider. It samples CPU, memory, swap,
  disks and disk I/O, network interfaces, processes, load average and
  uptime with `sysinfo`.
- Platform probes under `core/providers/sysinfo/platform/` add the values
  that `sysinfo` does not cover: governors, per-core temperatures, mount
  options, interface IPs, thread counts, process users, batteries, GPUs and
  Intel RAPL package power.
- `CompositeProvider` merges the primary provider with plugin data
  providers. The primary wins for non-empty data; extras fill the gaps.
- The process snapshot is sorted by CPU usage and capped by
  `XTOP_MAX_PROCESSES` (default 200).

Missing hardware is reported as an empty value or `None`. The core does not
fabricate readings.

## Snapshot model

The shared model lives in `xtop-plugin-api` and includes:

- `CpuInfo`, `MemoryInfo`, `SwapInfo`, `DiskInfo`, `DiskIOInfo`,
  `NetworkInfo`, `LoadAvg`, `BatteryInfo`, `GpuInfo` and `SystemInfo`.
- `ProcessInfo` with the extended process fields (command line, executable
  path, parent PID, start time, run time, users, groups, working directory,
  threads, file descriptors, disk totals and session id).
- `SystemSnapshot`, the per-tick aggregate consumed by the TUI and plugins.

## Live state and history

`core/state` owns `AppState`:

- The cached snapshot from the last tick.
- Bounded histories for per-core CPU, memory, network RX/TX, disk
  read/write and load.
- Per-process CPU history used by process widgets.
- Process search, sorting and selection state.
- Theme and layout selection.
- Alert thresholds, update interval and keybindings.
- Widget style resolution and the uid-to-name map.

History buffers are bounded by the configured `history_points`, so memory
stays flat during long sessions.

The state module also exposes shared view-control types (input mode,
fullscreen target, command palette state). TUI action dispatch and palette
behavior live in `src/tui`, not in `core`.

## Configuration

`core/config` owns the persisted schema and platform paths.

- `config.json` stores the theme, layout mode and name, update interval,
  history points, alert thresholds, keybindings, widget style and effect.
- Values are validated when loaded. The update interval is clamped to
  `100..=3_600_000` milliseconds to prevent a busy sampling loop.
- Missing or invalid files fall back to defaults; a partial write does not
  reset a running instance.
- The config directory is platform-aware: XDG on Linux, Application Support
  on macOS and AppData on Windows.

See [configuration.md](configuration.md) for the full schema.

## Themes

`core/theme` loads JSONC theme files and normalizes them for WCAG contrast.

- Only the base `x` theme is embedded. The other eleven themes are external
  catalog assets installed with `xtop theme download`.
- Theme files carry an explicit background and foreground plus a 16-slot
  palette with fixed roles.
- Contrast normalization happens in memory; user files are never rewritten.
- `core/config/persist.rs` exposes the persisted theme and layout to the CLI
  and the TUI run loop, which is how `xtop --change-theme` and
  `xtop layout set` reach running instances.

## Layouts

Layout definitions come from the `xtop-layout` crate and the user config
directory. Only the primary `Dashboard` layout is embedded; every other
layout is a JSONC file in the user layouts directory, installed from the
catalog with `xtop layout download`. `core/bootstrap::load_layout_defs`
merges the base layout with the user files, with user layouts replacing
built-ins by exact name.

The layout model and parser are not part of the kernel core; rendering the
tree is a TUI concern. `xtop layout check` uses the same parser without
starting the TUI.

## External asset catalog

`catalog.json` at the repository root lists every downloadable theme and
layout. Entries carry `id`, display `name`, `category` (layouts), `version`,
`author`, `license`, `file`, raw `url` and `sha256`. The base entries are
marked with `base: true`. The kernel keeps only the base `x` theme and
Dashboard layout embedded; everything else is fetched into the user config
directory.

`XTOP_CATALOG_URL` overrides the catalog source for mirrors, tests or local
files. Downloaded metadata is recorded in `assets.json` next to
`config.json`.

## Plugins and extensions

`core/plugins` hosts the plugin lifecycle.

- `PluginManager` registers plugins, creates per-plugin data directories,
  ticks them, dispatches keys, collects data providers and widgets,
  executes named actions and disables them on shutdown.
- `core/plugins/host.rs` implements the contract `HostState` for
  `AppState`, so plugins never depend on kernel types.
- `core/plugins/extension_host.rs` implements `ExtensionHost` and maps
  plugin errors to extension errors.
- Capability checks gate system reads, widget rendering and process
  termination.

Compile-time plugins are registered behind feature flags in
`core/bootstrap.rs`. Runtime widget hosts (WASM and helper processes) are
optional features and are discovered in the user config directory.

## Assets and bootstrap

`core/assets` owns the external asset lifecycle:

- `base.rs` embeds the `x` theme and `Dashboard` layout and creates them on
  first run or with `--gen-config`.
- `catalog.rs` parses and validates `catalog.json`, including safe file
  names, versions, authors, licenses and SHA-256 fields.
- `download.rs` fetches the catalog and assets (curl or local file),
  verifies checksums, validates JSONC, writes atomically and records the
  installed version in `assets.json`.
- `installed.rs` stores the local manifest used by `theme list` and
  `layout list`.

`core/bootstrap.rs` is the single assembly point used by the TUI and the
MCP command. It builds the provider, loads themes and layouts, constructs
`AppState`, registers plugins and collects plugin providers.

`core/state/persist.rs` persists the runtime-owned config fields on exit.

A headless consumer can reuse the same path:

```rust
let state = xtop::core::bootstrap::initialize_state(&config_dir)?;
```

## Data flow

1. The CLI parses arguments or starts the TUI.
2. The core bootstrap seeds assets and builds `AppState`.
3. On every tick the provider refreshes and returns one `SystemSnapshot`.
4. `AppState` caches the snapshot and appends history points.
5. Plugins tick and may contribute providers or widgets.
6. The TUI renders the active layout; the CLI can read the same state
   without rendering.

The render path never resamples the system. Every widget and plugin in a
frame sees the same snapshot.

## Platform coverage

| Signal | Linux | macOS | Windows | Fallback |
| --- | --- | --- | --- | --- |
| CPU usage and frequency | yes | yes | yes | empty |
| Per-core temperature | coretemp when the mapping is unambiguous | aggregate only | aggregate via thermal zones | none |
| Memory and swap | yes | yes | yes | empty |
| Disks and mount options | yes | yes | yes | partial |
| Network interfaces and IPs | yes | yes | yes | partial |
| Process fields | yes | yes | yes | partial |
| Batteries | sysfs | pmset | GetSystemPowerStatus | empty |
| GPU | nvidia-smi plus Linux sysfs | nvidia-smi | nvidia-smi | empty |
| Package power | Intel RAPL | none | none | none |

## Feature flags

Default:

```
plugin-samurai
mcp-extension
```

Optional:

```
widget-blocks
effects
plugin-wasm
plugin-external
```

`cargo build --no-default-features` builds the pure core without the
samurai plugin, the MCP extension, the blocks pack, effects or runtime
widget hosts. The core data model and providers are always present.

## Data honesty and safety

- Hardware data is never invented. Missing sensors, batteries or GPUs
  produce empty states.
- Plugin capability checks return `Result`; denied reads produce a
  recoverable error.
- External runtime widgets are not sandboxed beyond the process boundary;
  WASM widgets are sandboxed by fuel, memory and import limits.
- Process data includes an `environ` field for consumers. Do not log or
  publish environment contents.

## Building and testing

```
cargo build
cargo test
./scripts/ci.sh
```

`scripts/ci.sh` runs formatting, clippy, check, tests, a no-default build and
the structural audit. The audit covers file size, markers, re-exports,
per-area line budgets and the `core`/`cli`/`tui` dependency direction.

## Related documentation

- [commands.md](commands.md): every CLI command and option.
- [configuration.md](configuration.md): config schema and paths.
- [customization.md](customization.md): themes, layouts and runtime widgets.
- [plugin.md](plugin.md): plugin contract and authoring.
- [multi-repo.md](multi-repo.md): the wider ecosystem and repository split.
