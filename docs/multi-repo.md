# Multi-repo architecture (xtop-cli org)

> Status: **live** (2026-09-04). This document is the ecosystem architecture
> RFC location: it describes how the xtop-cli organization is split into
> repos, how the pieces depend on each other, and where the design is headed.
> Detailed per-area docs live in each repo's `docs/` folder; the push order
> and milestone state live in the root `ROADMAP.md` of the workspace.

## Organization

| Repo (xtop-cli) | Role | Content |
|---|---|---|
| `api` | **Contracts** | workspace with the four contract crates: `xtop-plugin-api` (data model, plugin/host traits, `AlertThresholds`, `PluginWidget`), `xtop-widget-api` (pack registration + glyph helpers), `xtop-extension-api` (extension host), `xtop-effect-api` (frame effects) |
| `xtop` | **Kernel** | the app: single-crate binary, `src/` by areas (commands, config, plugins, providers, state, theme, ui). Consumes every other repo |
| `widgets` | **Renderers** | packs of widget renderers against `xtop-widget-api`: base pack `xtop-widgets` + alternative pack `xtop-widget-blocks` (sibling crate of the per-widget crates so git consumers resolve its `xtop-widget-core` dependency) + `custom/` community |
| `layouts` | **Arrangement** | `xtop-layout` crate: data-driven layout model + JSONC loader + layout modes, plus `layouts/default/` (7 mode-bound layouts + 3 `detail_*` preset extras) and `layouts/custom/` (community, installable) |
| `plugins` | **Functionality** | plugin implementations against `xtop-plugin-api` (`xtop-plugin-samurai`), the runtime widget hosts (`xtop-plugin-wasm`, `xtop-plugin-external`) and their shared crates (`xtop-wasm-contract`, `xtop-widget-replay`, `xtop-wasm-guest`) |
| `extensions` | **Kernel hooks** | server-style extensions against `xtop-extension-api` (`xtop-extension-mcp`) |
| `effects` | **Animation** | frame effects against `xtop-effect-api` (`xtop-effect-fade`: 500 ms fade-in from black) |

Local development layout (sibling checkouts in one folder, as in this
workspace):

```
/home/x/xtop-cli/
  api/  xtop/  widgets/  layouts/  plugins/  extensions/  effects/
```

## Dependency principle: contracts in `api`, consumers point up

Each consumer repo depends only on the contract crates in `api` — never on
the kernel — so every repo compiles standalone:

```
                ┌────────────┐
                │    api     │  pure contract crates (ratatui/serde only)
                └─────┬──────┘
          ┌───────────┼───────────────┬──────────────┐
          ▼           ▼               ▼              ▼
   ┌──────────┐ ┌───────────┐ ┌────────────┐ ┌──────────────┐
   │  xtop    │ │ widgets/  │ │ layouts/   │ │ plugins/     │
   │ (kernel) │ │ effects/  │ │ extensions │ │ (samurai)    │
   └──────────┘ └───────────┘ └────────────┘ └──────────────┘
```

- **`api`**: pure types + protocols (manifests, capabilities, errors,
  snapshot model, provider/widget/extension/effect contracts). Depends only
  on `ratatui`/`serde`. Not published to crates.io yet (see Roadmap §6
  follow-ups).
- **`xtop` kernel**: hosts the ecosystem — implements `HostState` /
  `WidgetState` / `ExtensionHost` for its live state, runs `PluginManager`
  and the composite provider, renders layouts by resolving widget names into
  the packs, and drives effects (feature-gated). Builds fine without any
  optional feature: `cargo build --no-default-features` is the pure core.
- **`widgets` / `layouts` / `plugins` / `extensions` / `effects`**: consume
  only `api` types; each repo compiles standalone and never depends on the
  kernel.

## Integration modes

| Level | Mechanism | Use |
|---|---|---|
| Compile-time (today) | Cargo git dependencies + optional feature flags | Every integration: samurai plugin, mcp extension, blocks pack, fade effect |
| Dev-time | `xtop plugin install <name>` (clones, self-edits the kernel `Cargo.toml`, runs `cargo check`) | First steps with a plugin repo |
| Runtime widgets (today, opt-in) | Kernel features `plugin-wasm` (sandboxed `.wasm` modules via wasmi) and `plugin-external` (one helper process per widget over JSON lines), discovered in the config dirs `wasm/` and `external/` | Third-party widget code without recompiling the kernel — see `customization.md` ("Runtime Widgets") |
| Runtime (future, RFC) | binary/ABI discovery of `xtop-plugin-*` / `xtop-effect-*` / `xtop-extension-*` in config dirs + `XTOP_*_DEV_DIR` | Third parties without recompiling — see the root ROADMAP §7 deferred list |

The kernel never requires any external repo at runtime: optional ecosystem
pieces are Cargo features (`plugin-samurai`, `mcp-extension`,
`widget-blocks`, `effects`, `plugin-wasm`, `plugin-external`); contract crates
(the four `xtop-*-api` crates
plus `xtop-widgets`, `xtop-layout`) are unconditional because the kernel's
chrome and state views are written against them.

## Development flow (temporary path deps)

The ecosystem repos are edited before they are pushed. To compile a consumer
against un-pushed sibling state, its `Cargo.toml` temporarily replaces the
git dependency with a path dependency (`path = "../api/crates/plugin-api"`,
`path = "../widgets"`, ...), or — when the consumer's own manifest must stay
untouched — a temporary `[patch."https://github.com/xtop-cli/<repo>"]`
section redirects the git sources to the sibling checkouts. All temporary
overrides are removed before the owner pushes; final manifests carry the
floating git deps shown above.

## Why the split exists

The kernel was originally a workspace of kernel-owned crates
(`xtop-core`/`xtop-tui`/`xtop-cli`), then a monocrate with plugins inside.
Each customization axis grew into its own repo so that:

- a contributor can ship a widget pack, layout, plugin, extension or effect
  without touching the kernel code base;
- every repo compiles against the api contracts alone (testable in
  isolation, no kernel import);
- the kernel stays a thin host: metrics model, layouts and widget renderers
  all come from the ecosystem crates.

Single-source rules that keep the split honest (see the root ROADMAP
"decisions" section):

- the metrics model and plugin protocol exist only in `xtop-plugin-api`;
- widget registration and glyph/style mapping exist only in `xtop-widget-api`
  (the plugin-side widget is `xtop_plugin_api::PluginWidget`);
- ecosystem constants live at the producer (`xtop-plugin-samurai` exports
  `PLUGIN_ID` and its 12 action names; `xtop-extension-mcp` builds its tool
  table from them).

## Phases

1. **F0 (done)**: org `xtop-cli`, repos created, local clones in one folder.
2. **F1 (done)**: `api` contract crates extracted from the kernel's domain
   model; kernel + siblings consume them.
3. **F2 (done)**: samurai moved to the `plugins` repo; MCP moved to
   `extensions`; kernel features point at the sibling repos via git deps.
4. **F3 (done)**: `effects` workspace with the fade effect; kernel wires it
   behind the optional `effects` feature.
5. **F4 (done)**: extension host contract + `xtop-extension-mcp` as the
   first server-style extension.
6. **F5 (pending)**: publish the api crates to crates.io; tag and pin git
   deps; per-repo CI and kernel releases (see the root ROADMAP §6/§7).

## Open RFC topics

- Runtime dynamic discovery (ABI or directory-based loading) — explicitly
  deferred in the root ROADMAP §7 until the compile-time feature model stops
  being coherent.
- Publishing: versioning scheme for the contract crates and the pinning
  strategy of every consumer.
