# Xtop Roadmap

This document outlines the development roadmap for **xtop**, a modern, cross-platform TUI system monitor.
It is synced automatically with GitHub Issues.

## Phase 1: Core Features <!-- phase:core -->

- [x] Basic TUI structure using `ratatui` and `crossterm` (#1)
- [x] System information collection using `sysinfo` (#2)
- [x] CPU usage per core and maximum temperature sensing (#3)
- [x] Memory and Swap monitoring with historical graphing (#4)
- [x] Network upload and download tracking (#5)
- [x] Storage usage visualization (#6)
- [x] Top 50 process list sorted by CPU usage (#7)
- [x] Cross-platform compatibility (macOS, Linux, Windows) (#8)

## Phase 2: Interface & Theming <!-- phase:ui-themes -->

- [x] Dynamic layout manager supporting multiple modes (Dashboard, Vertical, Process Focus) (#9)
- [x] Implement 13 built-in color schemes (`x`, `madrid`, `tokio`, etc.) (#10)
      (12 theme files ship today, all embedded and seeded; the docs count
      12. See docs/colors.md for the palette reference.)
- [x] Instant theme and layout cycling at runtime (#11)
- [x] Responsive design for narrow terminals (#12)

## Phase 3: Deployment & Distribution <!-- phase:deployment -->

- [x] Automated install/uninstall scripts for Linux and macOS (`install.sh`, `uninstall.sh`) (#13)
- [x] Automated install/uninstall scripts for Windows (`install.ps1`, `uninstall.ps1`) (#14)
- [x] Quick curl/wget installation commands (#15)
- [x] CI/CD pipeline for automated multi-platform binary releases (#16)
- [ ] Distribution packages (AUR, Homebrew, Winget, APT) (#17)

## Phase 4: Configuration & Customization <!-- phase:config -->

- [x] Persistent configuration file support (save theme and layout preferences) (#18)
- [x] Custom user theme creation via configuration (#19)
- [x] Configurable update intervals for system metrics (#20)
- [x] Customizable keybindings (#21)

## Phase 5: Advanced Monitoring Features <!-- phase:advanced-monitoring -->

- [x] Disk I/O read/write speed tracking (#22)
- [ ] Granular network interface selection (#23) — per-interface RX/TX data is (#39)
      shown, but picking which interface the kernel reports/charts is not
      configurable yet.
- [ ] GPU usage, temperature, and VRAM monitoring (NVIDIA/AMD) (#24) — (#40)
      NVIDIA via nvidia-smi on any platform (Linux/macOS/Windows); AMD/Intel
      stays Linux-only partial (/sys/class/drm); macOS/Windows keep the
      honest empty state for AMD/Intel (no public utilization API).
- [ ] Battery status monitoring (#25) — Linux probes real (/sys/class/ (#41)
      power_supply); macOS real (pmset); Windows real aggregate battery
      (GetSystemPowerStatus; per-battery via SetupAPI pending); fallback
      platforms stay empty.
- [ ] Docker container resource usage integration (#26) — Docker support was (#42)
      removed from the shared data model (api M1.4: nothing consumed it), so
      this item is parked until a real consumer appears.

## Phase 6: Interactive Process Management <!-- phase:process-management -->

- [x] Interactive process termination (send kill signals) (#27)
- [ ] Search, filter, and highlight processes by name (#28) — search and (#43)
      filtering exist; highlighting matches inside the process list is
      pending.
- [ ] Tree view for process hierarchy (#29)
- [ ] Sorting processes by Memory, PID, or User (#30) — CPU/Memory/PID/Name (#44)
      sorting is implemented; no User column yet.

## Phase 7: X Integration <!-- phase:x-integration -->

- [ ] X Integration (#31)
- [ ] Create xp package (#32)
- [ ] Add to X Repositories (#33)

## Phase 8: Kernel Refactor <!-- phase:refactor -->

Baseline done: single-crate kernel (src/ by areas: config, theme, layout,
state, plugins, providers, ui, commands); plugin and extension hosts over
the api contracts; samurai and the mcp extension live in their own repos.
Only pending refactor work is listed here (git logistics are handled
outside this roadmap).

### R2 - Code quality pass

- [x] Module doc comments audit across src/ (#40) — the seven top-level (#45)
      modules declared in src/main.rs (commands, config, plugins, providers,
      state, theme, ui) all carry concise `//!` docs describing their area.
- [x] cfg(target_os) only inside platform/ trees (#41) — enforced by (#46)
      scripts/audit.sh (0 occurrences outside platform/).
- [ ] Wildcard re-export and pub hygiene review (#42) — 22 wildcard (#47)
      `pub use ...::*` re-exports remain (audit.sh threshold: 30); review
      still open.
- [x] Split commands/plugins.rs into list/install/scaffold modules (#43)
- [x] key_event_to_str into a shared input module if reused elsewhere (#44) — (#48)
      single use site (commands/run.rs), so no shared module is needed.
- [x] ui/share/error.rs only when real UI error handling appears (no empty (#49)
      modules) (#45) — intentionally not created; documented in the root
      ROADMAP deferred list.
- [x] Widgets subdivide internally when they outgrow one module (#46) — (#50)
      kernel widget renderers were externalized to the widgets repo (M3);
      the kernel no longer owns pack widgets.

### R3 - Structural audit tooling

- [x] scripts/audit.sh with failing thresholds (#47) — the script gates: (#51)
      cfg(target_os) outside platform/ trees = 0, files over 600 lines = 0,
      TODO/FIXME/XXX/HACK markers = 0, wildcard `pub use ...::*` <= 30,
      LOC per top-level area <= 2400, dead pre-monocrate plugin tree absent
      (its path lives only inside the audit script as the guard itself),
      `miami` embedded in the theme seeds (12 themes total). Module
      dependency graph/cycles and unused-pub detection are NOT implemented
      by the script (see the deferred note below).

Deferred follow-ups (tracked with the root ROADMAP §7 list):
- audit.sh module dependency graph/cycles and unused-pub detection, plus
  per-file thresholds above 300/600 lines (the 200-line figure in the
  original issue predates the current 300/600 gates).
