# Changelog

## [0.1.0] - 2026-09-04

Version policy note: the ecosystem stays early — everything is 0.1.0. The
entry below consolidates the post-refactor kernel work of this cycle
(theme format v2 + contrast engine, preset extras, sort direction,
role-colored chrome, ecosystem externalization, data surface, density and
legibility) into one cumulative list.

### Kernel data surface
- Theme format v2: the 12 shipped theme files carry explicit
  `background`/`foreground` keys from the canonical owner palettes
  (legacy files fall back to slot0/slot7); seed bumped 4 → 5.
- Contrast engine (WCAG luminance): at load, text roles (fg, dim,
  accent, zebra, series marks) are normalized against the theme
  background with hue-preserving lifts when below floor; regression
  pinned (Paris fg is never palette[7] when it equals the background).
- Data surface: `CpuInfo.temp_c` (Linux coretemp probe fills per-core
  temps where unambiguous; macOS/Windows/fallback stubs return None) and
  bounded WidgetState histories for aggregate disk read/write rates and
  load average.
- `CpuInfo.frequency`, `.governor`, `.name` (provider.rs) and other
  collected metrics (state, run_time, threads, open files, disk bytes,
  exe_path, cmd, cmd_full, parent_pid) keep flowing through the snapshot
  for widgets and plugins.
- Dense widgets: boxes scale charts to full height; cpu per-core temp
  view (`show_temp`); memory available row; network/disk_io multi-row
  dual braille charts; new `summary` and `sensors` widgets (both packs);
  `detail_*` presets rebuilt as full-monitor pages with zero orphan rows
  at common sizes (tiling rules documented).

### Presets, chrome and layout extras
- The three preset extras are named `detail_*` (file names
  `detail_dashboard`, `detail_network`, `detail_processes`; display names
  `Detail Dashboard`, `Detail Network`, `Detail Processes`). Help overlay,
  docs, tests and comments updated; seed version bumped 3 → 4 so existing
  installs re-seed the renamed layout templates (old seeded files are left
  untouched, per the never-clobber policy).
- Layout presets as extras (DR-UX6): the kernel embeds the three reference
  preset layouts from `xtop-cli/layouts` after the seven mode-bound layouts;
  the layout cycling key reaches them in file order and wraps around, the
  command palette lists them by name, and first-run asset seeding copies
  them as editable templates (seed version 2 → 3).
- The UX-cycle design document is reworked into `docs/design.md`: a neutral
  statement of xtop's own design rules (theme roles, glyph sets,
  per-widget display options, minimum-width fallbacks, chart color rules,
  deferred features), with the recognized per-widget option keys
  cross-referenced to the widgets repo docs. All links updated.
- Kernel chrome polish (UX5 carry-over, gaps closed): the search overlay
  now draws its border with the configured `style.borders` glyph set like
  the help and palette popups; titles keep the accent role, separators the
  dim role, and no hardcoded double-line borders remain in kernel chrome.
- Sort-direction cycle and per-widget display options are carried over
  unchanged from the same cycle; widget-side rendering changes (visual
  language v2) land in the widgets repo changelog notes.

### Process sort, command palette, role colors
- Process sort direction: the sort key (`s`, configurable) alternates
  between flipping the order on the current column (boot default: CPU%
  descending, unchanged) and advancing to the next column (CPU% → Mem
  → PID → Name), which starts descending — one press flips `▲`/`▼`, the
  next press moves on. The order actually rendered honors the direction,
  and `process_sort_desc()` (new additive `WidgetState` method, default
  `false`) lets the processes widget draw the column marker.
- Palette "Sort" entry shows the current column with its direction marker.
- Role-colored kernel chrome (DR-UX3): new theme role accessors
  (`accent()` = slot 6, `dim()` = slot 8, documented in `docs/colors.md`);
  help overlay and search/palette popups use accent titles/borders, dim
  separators and the configured `style.borders` glyph set (help no longer
  hardcodes double-line borders); minimal-view gauges keep their role
  colors with the roles documented.
- Help overlay text updated: sort-direction behavior and the preset layouts
  are listed; the "Layouts" section reflects modes → presets → user
  layouts.

### Ecosystem externalization (monocrate cycle complete)
- The kernel is now a thin host over the externalized ecosystem. It
  consumes the contract crates from `xtop-cli/api` (`xtop-plugin-api`,
  `xtop-widget-api`, `xtop-extension-api`), the widget packs from
  `xtop-cli/widgets`, layouts from `xtop-cli/layouts`, the samurai plugin
  from `xtop-cli/plugins` and the MCP extension from `xtop-cli/extensions`
  as floating git dependencies with optional feature flags.
- Deleted the dead uncompiled `src/commands/plugins_dir_tmp/` leftover
  from the pre-monocrate layout. (Path spelled in scripts/audit.sh as the
  absence guard.)
- `rust-version = "1.87"` declared.

### Local-only CI
- The GitHub Actions CI workflow (`.github/workflows/ci.yml`) has been
  removed: gating is local only (`scripts/ci.sh`, `scripts/audit.sh`) and
  will not be re-enabled until the release pipeline is 100% ready.

### Contract consolidation
- `config::AlertThresholds` removed: the persisted config now uses
  `xtop_plugin_api::AlertThresholds` directly (identical JSON keys
  `cpu_high`/`mem_high`/`disk_high`; defaults 90/90/90 constructed at the
  config layer). Hand-marshalling bridge code deleted from
  `plugins/host.rs` and `state/widget_state.rs`.
- Plugin widget registrations use `xtop_plugin_api::PluginWidget`; the
  duplicated `PluginWidgetFn` alias in `ui/layout/engine.rs` is gone.
- Docker leftovers removed (`dockers: vec![]` assignment, `docker_info`
  composite forwarding) — the api model no longer carries Docker data.
- Plugin context reads are capability-gated and `Result`-typed upstream;
  consumers adapted.

### Themes
- `miami` is now embedded and seeded like the other 11 themes (12 JSONC
  theme files ship, all in `DEFAULT_THEMES`); seed version bumped to "2"
  so existing installs receive the new template.
- Theme docs/counts corrected everywhere (12 themes; `x` compiled in as
  the startup fallback).
- `theme::themes_dir()` routes through the platform-aware
  `config::config_dir()`, so themes live next to `config.json` and layouts
  on macOS/Windows too.

### Dependency refresh
- ratatui `0.29 -> 0.30.2` (`Layout::vertical`/`Layout::horizontal`
  builder API); crossterm aligned with ratatui's backend (single
  `crossterm 0.29` in the graph via `ratatui-crossterm`).

### New features
- Unknown widget names in the active layout produce a one-time stderr
  warning per name (`xtop: layout '<layout>' references unknown widget
  '<name>'`).
- Optional `effects` feature (off by default) wires the built-in
  `xtop-effect-fade` through the `effect` config key ("fade" activates the
  500 ms fade-in; absent/unknown disables). Builds without the feature
  carry zero extra dependencies.

### Docs and release plumbing
- `docs/` refreshed: configuration keys, plugin architecture (plugins as
  separate crates, `plugin list|install|scaffold` real behavior, MCP
  extension), multi-repo RFC status, feature/theme counts; `colors.md`
  moved under `docs/`.
- `docs/colors.md` palette role table corrected to actual code usage
  (slots 4/5 are the RX/TX read/write pair, not "storage/TX"); per-widget
  display options documented in `docs/customization.md`; layout
  counts/cycles updated across the docs.
- `install.sh` VERSION synced to the crate version; OpenSSL build-dep
  claims removed (the crate has no openssl dependency); `install.ps1`
  placeholder comment removed.
- Kernel `ROADMAP.md` synced with the implemented state (phases 4-6 and
  the R2/R3 refactor items); `scripts/audit.sh` extended with dead-dir and
  theme-seed gates.
- Crate version pinned to `0.1.0` (installer `VERSION` synced) per the
  ecosystem version policy: stay early, no more bumps.

### Runtime widget hosts (opt-in)
- New optional features `plugin-wasm` (sandboxed `.wasm` widgets via wasmi)
  and `plugin-external` (one helper process per widget over line-delimited
  JSON); both are non-default and absent from the `default` set, so the
  compiled-in plugin/pack system and a plain build are unchanged. Runtime
  widgets register through the plugin widget path (precedence over every
  pack), are discovered in `<config dir>/wasm/` and `<config dir>/external/`
  (`XTOP_WASM_DIR` / `XTOP_EXTERNAL_DIR`) and are referenced in layouts by
  the manifest `name`.
- Hosts and shared crates live in `xtop-cli/plugins` (`xtop-plugin-wasm`,
  `xtop-plugin-external`, `xtop-wasm-contract`, `xtop-widget-replay`,
  `xtop-wasm-guest`) and are consumed as git dependencies, so a clean clone
  builds without sibling checkouts.
- Installers can enable the hosts: `install.sh --with-runtime-widgets` and
  `install.ps1 -RuntimeWidgets` pass `--features
  plugin-wasm,plugin-external` to the build; the default install stays
  unchanged.
- `--all-features` now also enables the runtime hosts (wasmi plus process
  spawning); `docs/installation.md` states this.
- Docs: `docs/customization.md` and `docs/plugin.md` gained a "Runtime
  Widgets" section; `docs/multi-repo.md` integration modes updated. The
  rationale is recorded as ADR-001 in the plugins repo
  (`plugins/docs/decisions.md`).

### Theme switching from the CLI
- `xtop --ct <theme>` changes the active theme from the command line and
  persists it to `config.json`; a running instance follows the change live
  (the TUI run loop polls the persisted theme at tick boundaries), so
  external themers such as a Hyprland theme switcher can drive it without
  IPC. Unknown names fail with the list of available themes
  (`src/commands/theme.rs`, `docs/usage.md`, `docs/customization.md`).

## Earlier history

### [0.0.1] - 2026-06-18

#### Config: Persistencia de Layouts Personalizados
- `Config` ahora tiene campo `layout_name` que almacena el nombre del layout seleccionado (soporta layouts mas alla de los 7 built-in `LayoutMode`)
- En configuraciones existentes, `layout_name` es opcional (`#[serde(default)]`) y se usa como respaldo `layout_mode`
- `save_config` guarda el nombre del layout actual por su indice en `layout_defs`, permitiendo restaurar layouts personalizados al reiniciar
- `AppState::new` prioriza `config.layout_name` si no es vacio, con fallback a `layout_index_from_mode`
- Config se guarda automaticamente al seleccionar tema o layout desde la paleta de comandos

#### Paleta de Comandos: Soporte macOS
- Agregada ruta directa para `ctrl+p` en el bucle de eventos, independiente del sistema de keybindings (soluciona problemas donde crossterm reporta la tecla con distinto casing o el keybinding no se resuelve)
- `key_event_to_str` aplica `to_ascii_lowercase()` al caracter cuando Ctrl esta activo, normalizando `"ctrl+P"` a `"ctrl+p"`
- Agregado `"ctrl+P"` como variante alternativa en el keybinding por defecto
- Debug output de teclas en compilaciones debug (`cargo build` sin `--release`) para diagnosticar problemas: `[key] 'ctrl+p'`

#### Plugins: Seguridad y Arquitectura
- `PluginContext` ahora verifica capabilities antes de ejecutar acciones sensibles: `kill_process`, `set_alert_thresholds`, `set_theme_by_name`, `set_layout_by_name`, `set_update_interval` requieren `KillProcesses` o `ModifyConfig` segun corresponda
- `PluginCapability::Custom` migrado de `&'static str` a `String` para flexibilidad
- `PluginManifest` usa `String` en vez de `&'static str`, permitiendo plugins que generen metadata dinamicamente
- `#[non_exhaustive]` agregado a `PluginCapability` para evolucion segura del enum

#### PluginManager: Robustez
- `with_plugin_manager_mut()` reemplaza el patron inseguro `take()` + `Some()` que podia perder el manager si una ruta de error no lo restauraba. Todos los callers migrados: `on_tick`, manejo de teclas, shutdown, y MCP server
- `register()` ahora devuelve `Result<(), PluginError>` en vez de `Result<(), String>`, consistente con el resto del sistema
- Metodo `build_context()` elimina la duplicacion de construccion de `PluginContext` en 5 metodos distintos
- `data_dir` del plugin ahora apunta al directorio del plugin (`plugins/<id>/`), no a `plugins/<id>/config.json`

#### Capabilities: Validacion Real
- `collect_widgets()` solo acepta widgets si el plugin declara `RenderWidgets`
- `collect_data_providers()` solo acepta providers si el plugin declara `ReadSystemInfo`
- `PluginContext` inyecta las capabilities declaradas y verifica en cada metodo sensible

#### MCP Server
- Migrado de `take()`/`Some()` a `with_plugin_manager_mut()` para seguridad
- Eliminado doble `tick_all()` innecesario en el handler de herramientas

#### Sistema de Providers
- `SystemDataProvider` ahora tiene metodo `add_extras()` con default no-op, eliminando la necesidad de `downcast_mut::<CompositeProvider>()`
- `kill_process()` verifica que el UID del proceso coincida con el usuario actual antes de enviar SIGTERM
- Limite de procesos configurable via `SysinfoProvider::max_processes` (default 200)
- Eliminados `NoopBatteryProvider`, `NoopGpuProvider`, `NoopDockerProvider` — codigo muerto no utilizado

#### Sentinel: Calidad de Datos
- Migracion completa de construccion manual de JSON (`format!` con strings escapados) a `serde_json::json!()` — elimina riesgo de inyeccion JSON y corrupcion por caracteres especiales

#### TUI
- `LayoutConstraint::Fill` mapeado correctamente a `ratatui::Constraint::Fill(1)` en vez de `Min(0)`

#### Infraestructura
- `config_dir()` centralizada en `xtop_core::infrastructure::config::config_dir()` — eliminada duplicacion en 5 modulos
- CI workflow creado en `.github/workflows/ci.yml`: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

### [0.2.0] - 2026-06-03

#### Refactorizacion Total del Proyecto
- Migrado a workspace multi-crate: `xtop-core`, `xtop-tui`, `xtop-cli`
- Eliminado el monolito `src/` — ahora cada capa vive en su propio crate
- Eliminadas 5 dependencias muertas: `serde`, `serde_json`, `clap`, `chrono`, `tokio` (se redujo de ~84 a ~55 crates)
- Eliminado codigo muerto: `InputMode`, `show_help`, `swap_history`, `process_table_state`, `graph_colors()`
- Eliminados todos los `#[allow(dead_code)]`

#### Nueva Arquitectura Hexagonal
- Capa de Dominio (`xtop-core/domain/`): modelos de datos puros + trait `SystemDataProvider`
- Capa de Aplicacion (`xtop-core/application/`): `AppState`, `MetricsHistory` (con `VecDeque`), `LayoutMode`, `EffectiveLayout`
- Capa de Infraestructura (`xtop-core/infrastructure/`): `SysinfoProvider`, `theme_loader`, `config`, providers stub
- Capa de Presentacion (`xtop-tui/`): terminal, render widgets separados, format helpers
- Binary (`xtop-cli/`): entry point con inyeccion de dependencias

#### Layout Responsive
- `detect_effective_layout(width, height, mode)` adapta el layout automaticamente:
  - **Dashboard** (>100x30): layout completo 2-columnas
  - **Compact** (>80x24): mas compacto
  - **Vertical** (<80): todo apilado
  - **Minimal** (<60 ancho o <18 alto): solo CPU + Mem + procesos
  - **Too Small** (<40x8): mensaje de advertencia

#### Nuevos Layouts (7 modos, ciclo con `l`)
| Modo | Descripcion |
|------|-------------|
| Dashboard | Default, 2-columnas con graficos |
| Vertical | Apilado, para terminales estrechas |
| Horizontal | 4 columnas: CPU/Mem/Storage/Network |
| CPU Focus | CPU grande + procesos |
| Memory Focus | Memoria grande con chart + procesos |
| Network Focus | Network + Disk I/O lado a lado + procesos |
| Process Focus | Stats pequenos + procesos maximizados |

#### Full Screen (`f` / `F`)
- `f` activa/desactiva modo fullscreen
- `F` cicla entre widgets (CPU, Memory, Storage, Network, Processes, Disk I/O, GPU, Battery, salir)
- Widget seleccionado ocupa toda la terminal (menos header)

#### Busqueda de Procesos (`/`)
- Filtrado en tiempo real por nombre de proceso
- `Enter` confirma el filtro, `Esc` cancela, `Backspace` borra
- Overlay centrado con indicador `/query_`

#### Ayuda en Pantalla (`?`)
- Muestra todas las keybindings disponibles
- Cierra con `Esc` o `?` otra vez

#### Nuevas Metricas
- **Disk I/O**: velocidad de lectura/escritura por disco (bytes/s) con widget dedicado
- **Per-interface Network**: RX/TX y velocidad por interfaz de red
- **GPU**, **Battery**, **Docker** stubs preparados para implementacion futura

#### Alertas por Threshold
- **CPU > 90%**: color cambia a rojo
- **Memoria > 90%**: color rojo + icono de advertencia en el titulo
- Thresholds configurables en `AlertThresholds` (cpu_high, mem_high, disk_high)

#### Mejoras de Codigo
- `Vec` + `remove(0)` reemplazado por `VecDeque` con `pop_front()` (O(1))
- Helper `format_bytes()` elimina repeticion de `1024.0 / 1024.0 / 1024.0`
- Helper `format_uptime()` para formato legible de tiempo activo
- `MetricsHistory::set_max_points()` para configurar puntos del historico

#### Configuracion Persistente
- `~/.config/xtop/config.json`: guarda tema, layout, intervalo, history_points, alerts
- `~/.config/xtop/themes/*.json`: temas personalizados por el usuario
- Guardado automatico al salir con `q`
- Temas built-in (13) se fusionan con temas personalizados

#### Tests
- 39 tests unitarios (de 0): layout detection, history, themes, format helpers, config
- CI workflow `.github/workflows/ci.yml`: check, fmt, clippy, test, build

#### Keybindings Completos
| Tecla | Accion |
|-------|--------|
| `q` | Salir (guarda config) |
| `?` | Ayuda |
| `t` / `T` | Siguiente/anterior tema |
| `l` | Siguiente layout |
| `f` / `F` | Toggle fullscreen / ciclar widget |
| `/` | Buscar procesos |
| `Esc` | Cancelar busqueda / cerrar ayuda |
