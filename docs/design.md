# Xtop design language

Scope note for readers: this document states the visual design rules xtop
renders by (the 0.1.0 kernel + the widget packs in their UX working state,
2026-09-04): theme roles, glyph sets, per-widget display options,
minimum-width fallbacks and chart color rules. Widget-side claims are
cross-checked against the widgets repo working tree of the same date; the
widgets agent's final report (`tmp/ux2-4-widgets-report.md`, if present) is
authoritative for per-widget details and takes precedence over the summary
claims here.

The language is informed by btop-class resource monitors; the reference
notes collected during the UX0 research live under `tmp/` in the workspace
root.

## Rules

| Rule | Where it is implemented |
|---|---|
| Chrome comes only from theme roles (DR-UX3): one role table in `docs/customization.md` / `docs/colors.md`; kernel chrome (help/palette/search popups) uses accent titles/borders + dim separators and follows the configured border set (`style.borders`, incl. pure ASCII `+-|`); widget frames draw accent/dim roles (`widgets/src`, `ROLE_*`) | Kernel `src/tui/overlay/*`, `src/tui/screen.rs`; widget packs |
| Sort state is always visible: the process column that sorts carries a colored marker, one press flips the direction (`▼` ↔ `▲`), the next press advances CPU% → Mem → PID → Name (boot default = CPU% descending, classic order preserved) | Kernel `AppState::cycle_sort` + `process_sort_desc()` on `WidgetState`; the processes widget paints the marker from that state |
| Per-widget display discrimination is data-driven from the layout file (`options` on widget nodes, DR-UX1), not from global config only — two instances of the same widget in one screen can render differently | Layouts repo (`docs/layout-schema.md`, "Widget `options`"); kernel `WidgetState::widget_options`; recognized keys per widget in the widgets repo docs |
| Row selection survives re-sorts: the selected process row is PID-anchored and highlighted (accent background/inverted row); zebra rows and column separators use the dim role | Processes widget (`widgets/`) + kernel selection state |
| Chart markers and borders honor the glyph set chosen per widget (charset: braille/dot/block/half_block/bar; borders: native/rounded/double/plain/ascii), with global defaults and per-widget overrides in `config.json` under `style`; per-layout-node `options` can refine them per instance | `ChartCharset`/`border_for` in the widget-api contract; widget packs; kernel passes `style.widgets.<name>` through |
| Data series get role-paired colors: per-core rows use a gradient fill, the alert/warn/good roles drive status coloring, and multi-series lines (RX/TX, read/write) use the documented slot pair (RX = slot 4, TX = slot 5) | Widget packs; role legend in `docs/colors.md` |
| Small terminals degrade cleanly instead of wrapping: layout modes fall back by size (below 60×14 → minimal gauges + process list; Dashboard compacts below 100×28 and becomes vertical below 80 columns), with a hard floor of 40×8 | `xtop-layout` `mode.rs` (`detect_effective_layout`), kernel `tui/screen.rs` |
| Display options never fabricate data: every option refines how existing metrics are drawn (bases, columns, cores, interfaces, disks, units); underlying metrics are always real | Widget packs + data providers |
| Layout presets are named extras, not modes: Dashboard is embedded, the other mode layouts and the `Detail Dashboard`, `Detail Network` and `Detail Processes` presets are catalog assets; cycling follows the installed file order, then user/community layouts (DR-UX6) | Layouts repo `src/loader.rs`; kernel layout cycling + `xtop layout download` |

## Recognized per-widget option keys

The layout `options` passthrough is interpreted by the widget renderers;
each widget documents the keys it recognizes in the widgets repo
(`docs/widgets.md`). The shipped `detail_*` presets exercise the first wave
on the `processes`, `cpu` and `network` widgets (see the layouts repo
schema doc for the exact objects they carry).

## Deferred (deliberately not modeled)

These reference-monitor features have no xtop equivalent yet; each line
states the data or architecture reason, so nothing here is stubbed:

- Per-process detail panel (graphs, status, IO, parent, cmd, …): per-process
  *CPU* history exists since UX9.1 (`WidgetState::process_cpu_history`, a
  bounded per-pid ring fed from the visible process list), so a small CPU
  spark per row is drawable; no status/IO/parent history buffers exist yet.
  Process metadata (state, parent, cmd, thread count, open files, disk
  bytes) is already in the model and available to future views.
- Tree process view with expand/collapse prefixes: no tree semantics in the
  process view contract; PID anchoring assumes a flat, sortable list. Needs
  a model decision before UI work.
- Follow mode + paused/following banners: no follow concept in the kernel
  state machine (`InputMode` is Normal/Searching/CommandPalette). Banners
  would need view-state additions.
- Per-core temperatures: since UX8.3 the provider collects per-core
  temperatures on Linux (`CpuInfo::temp_c`, coretemp sensors) when the
  sensors map onto the logical cores; machines without a readable mapping
  keep the aggregate max (`cpu_temp`) only, so no per-core temp row is
  drawn there.
- 101-step gradient ramps per metric family: the 16-slot palette cannot
  host per-family 101-step ramps. xtop uses a documented 3-stop gradient
  (alert/warn/good roles) and the bright ramp for multi-series lines.
- Graph overlay captions (uptime on the CPU graph, net scale text):
  uptime/load live in the header widget; chart y-scales are drawn on the
  chart axis. Overlay captions would need a graph-text style role the
  palette does not define.
- Transparent background and 16-color downconvert: the palette background
  role is always painted; there is no SGR 16-color path. Both are kernel
  color-mode features, deferred.
- Mouse-driven chips / inline hotkey buttons: input is keyboard-first
  (help overlay via `?`); mouse support is limited to wheel scrolling of
  the process list.
- Runtime per-box show/hide toggles: layout switching covers box selection
  (`l`, palette, config), but runtime per-box show/hide toggles are not
  modeled in the layout engine.
- Fixed network chart ceilings: charts auto-scale to the sampled maximum;
  configurable fixed ceilings are not implemented.
- Kill confirmation dialog: `k` kills the PID-anchored selection directly;
  no dialog/banner exists (visual feedback is the row disappearing).

## Where xtop goes further

- Widget packs are swappable per widget name and per layout instance
  (`style.widgets.<name>.pack`), so a single binary ships multiple visual
  languages for the same metric.
- Display discrimination is data-driven from the layout file (`options` on
  widget nodes), not from global config only — two instances of the same
  widget in one screen can render differently.
- The kernel is provider-driven; data comes from the sysinfo provider and
  plugins, so detail views were only built where the data truly exists.

## Related documents

- `docs/colors.md` — theme palettes + role legend; `docs/customization.md`
  — the single role table, per-widget `options`, layout presets.
- Widgets repo — `docs/widgets.md`: recognized per-widget option keys.
- Layouts repo — `docs/layout-schema.md`: the `detail_*` preset layouts and
  the `options` passthrough grammar.
