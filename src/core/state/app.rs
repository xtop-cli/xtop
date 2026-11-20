//! Live application state: system sampling per tick, layout/theme/process
//! control, plugins.

use crate::core::config::keybinding::Keybindings;
use crate::core::config::{Config, UiStyle};
use crate::core::plugins::PluginManager;
use crate::core::state::history::MetricsHistory;
use crate::core::state::proc_history::ProcessCpuHistory;
use crate::core::state::users::Users;
use crate::core::state::view::{
    FullScreenWidget, InputMode, PalettePage, PaletteState, ProcessSortBy,
};
use crate::core::theme::Theme;
use xtop_layout::{layout_index_from_mode, layout_mode_for_name, LayoutDef, LayoutMode};
use xtop_plugin_api::model::{ProcessInfo, SystemInfo, SystemSnapshot};
use xtop_plugin_api::{AlertThresholds, PluginWidget, SystemDataProvider};

pub struct AppState {
    provider: Box<dyn SystemDataProvider>,
    pub history: MetricsHistory,
    /// Per-process CPU samples for braille sparks (UX9.1): bounded map fed
    /// every tick from the visible process list.
    pub proc_cpu_history: ProcessCpuHistory,
    /// Uid → login-name table for the processes view (UX9.1), from
    /// `/etc/passwd` on unix (empty elsewhere → numeric fallback).
    pub(crate) users: Users,
    pub should_quit: bool,
    pub layout_mode: LayoutMode,
    pub layout_index: usize,
    pub layout_defs: Vec<LayoutDef>,
    pub current_theme: Theme,
    pub themes: Vec<Theme>,
    pub selected_theme_index: usize,
    pub tick_count: f64,
    pub show_help: bool,
    pub input_mode: InputMode,
    pub search_query: String,
    pub full_screen_widget: FullScreenWidget,
    pub alerts: AlertThresholds,
    pub update_interval_ms: u64,
    /// Widget glyph style (chart charset, borders); from the user config.
    pub style: UiStyle,
    pub palette: PaletteState,
    pub keybindings: Keybindings,
    pub process_sort: ProcessSortBy,
    /// Direction of the active process sort: `true` = descending (largest
    /// first). Boots descending to keep the classic CPU%/Mem order; the sort
    /// key toggles it per column before advancing (see [`AppState::cycle_sort`]).
    pub process_sort_desc: bool,
    /// Selected process anchored by PID (not row index), so sorting/filtering
    /// or a fresh sample never makes a kill target the wrong process.
    pub process_selected_pid: Option<u32>,
    pub sys_info: SystemInfo,
    /// Latest full system sample, computed once per tick and shared by every
    /// widget/action in that frame (avoids N samples per frame).
    last_snapshot: Option<SystemSnapshot>,
    pub plugin_manager: Option<PluginManager>,
    pub plugin_widgets: Vec<PluginWidget>,
    /// Layout `options` of the widget currently being rendered (DR-UX1).
    ///
    /// The render engine sets this right before each pack-widget render call
    /// (from the layout node that named the widget) and resets it to `None`
    /// right after; the `xtop_widget_api::WidgetState::widget_options`
    /// implementation reads it. It is `None` outside a render call and for
    /// widget instances whose layout node carries no `options` object, so
    /// behavior is byte-identical to the pre-DR-UX1 rendering. Plugin widget
    /// renderers see `HostState` (not `WidgetState`) and never observe this
    /// field; it is kernel-internal plumbing and never persists across
    /// frames.
    pub(crate) active_widget_options: Option<serde_json::Value>,
}

impl AppState {
    pub fn new(
        provider: Box<dyn SystemDataProvider>,
        themes: Vec<Theme>,
        config: Config,
        layout_defs: Vec<LayoutDef>,
    ) -> Self {
        let selected_theme_index = themes
            .iter()
            .position(|t| t.name == config.theme)
            .unwrap_or(0);
        let current_theme = themes[selected_theme_index].clone();
        let layout_index = if !config.layout_name.is_empty() {
            layout_defs
                .iter()
                .position(|l| l.name == config.layout_name)
                .unwrap_or_else(|| layout_index_from_mode(config.layout_mode, &layout_defs))
        } else {
            layout_index_from_mode(config.layout_mode, &layout_defs)
        };
        Self {
            provider,
            history: MetricsHistory::new(config.history_points),
            proc_cpu_history: ProcessCpuHistory::new(),
            users: Users::load(),
            should_quit: false,
            layout_mode: config.layout_mode,
            layout_index,
            layout_defs,
            current_theme,
            themes,
            selected_theme_index,
            tick_count: 0.0,
            show_help: false,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            full_screen_widget: FullScreenWidget::None,
            alerts: config.alerts,
            update_interval_ms: config.update_interval_ms,
            style: config.style,
            palette: PaletteState {
                open: false,
                query: String::new(),
                selected: 0,
                entries: Vec::new(),
                filtered: Vec::new(),
                page: PalettePage::Main,
            },
            keybindings: config.keybindings,
            process_sort: ProcessSortBy::Cpu,
            process_sort_desc: true,
            process_selected_pid: None,
            sys_info: SystemInfo::default(),
            last_snapshot: None,
            plugin_manager: None,
            plugin_widgets: Vec::new(),
            active_widget_options: None,
        }
    }

    /// Set the plugin manager and inject extra data providers into the composite provider.
    /// Called once during initialization after all plugins are registered.
    pub fn init_plugins(
        &mut self,
        mgr: PluginManager,
        extra_providers: Vec<Box<dyn SystemDataProvider>>,
    ) {
        if !extra_providers.is_empty() {
            self.provider.add_extras(extra_providers);
        }
        self.plugin_manager = Some(mgr);
        self.refresh_plugin_widgets();
    }

    /// Collect plugin widgets into the state for the TUI renderer.
    pub fn refresh_plugin_widgets(&mut self) {
        if let Some(ref mgr) = self.plugin_manager {
            let registrations = mgr.collect_widgets();
            // Only accept if the plugin has RenderWidgets capability
            self.plugin_widgets = registrations;
        }
    }

    /// Kill a process by PID. Returns true if the signal was sent.
    pub fn kill_process_by_pid(&mut self, pid: u32) -> bool {
        self.provider.kill_process(pid)
    }

    /// Set alert thresholds.
    pub fn set_alert_thresholds(&mut self, cpu: f64, mem: f64, disk: f64) {
        self.alerts = AlertThresholds {
            cpu_high: cpu,
            mem_high: mem,
            disk_high: disk,
        };
    }
    /// Switch to a theme by name. Returns true if found.
    pub fn set_theme_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.themes.iter().position(|t| t.name == name) {
            self.selected_theme_index = idx;
            self.apply_theme();
            true
        } else {
            false
        }
    }

    /// Switch to a layout by name. Returns true if found.
    pub fn set_layout_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.layout_defs.iter().position(|l| l.name == name) {
            self.layout_index = idx;
            self.sync_layout_mode();
            self.full_screen_widget = FullScreenWidget::None;
            true
        } else {
            false
        }
    }

    pub fn current_layout(&self) -> &LayoutDef {
        &self.layout_defs[self.layout_index]
    }

    /// The layout mode matching the current definition. Custom layouts fall
    /// back to the previously active mode (they are addressed by name).
    ///
    /// External layouts make index-based mode inference invalid: only an
    /// explicit built-in name selects a mode, everything else keeps the
    /// previous mode.
    pub fn save_layout_mode(&self) -> LayoutMode {
        match self.layout_defs.get(self.layout_index) {
            Some(def) => layout_mode_for_name(&def.name, self.layout_mode),
            None => LayoutMode::Dashboard,
        }
    }

    pub(crate) fn sync_layout_mode(&mut self) {
        self.layout_mode = self.save_layout_mode();
    }

    /// Safely access the plugin manager with a closure.
    /// Ensures the plugin manager is always restored after the operation.
    /// Returns `None` when no manager is initialized (pre-bootstrap or tests)
    /// instead of panicking; callers decide how to degrade.
    /// NOTE: does NOT call refresh_plugin_widgets — the caller must do it if needed.
    pub fn with_plugin_manager_mut<R>(
        &mut self,
        f: impl FnOnce(&mut PluginManager, &mut Self) -> R,
    ) -> Option<R> {
        let mut mgr = self.plugin_manager.take()?;
        let result = f(&mut mgr, self);
        self.plugin_manager = Some(mgr);
        Some(result)
    }

    pub fn on_tick(&mut self) {
        self.provider.refresh_all();
        self.tick_count += 1.0;
        let info = self.provider.system_info();
        if !info.hostname.is_empty() {
            self.sys_info = info;
        }
        let x = self.tick_count;
        let snap = self.provider.snapshot();

        if snap.cpus.len() != self.history.cpu.len() {
            self.history.reset_cpu(snap.cpus.len());
        }
        for (i, cpu) in snap.cpus.iter().enumerate() {
            self.history.push_cpu(i, x, cpu.usage);
        }

        self.history.push_mem(x, snap.memory.percent);

        // Network history tracks *rates* (bytes/s), not cumulative counters,
        // so the chart shows throughput over time.
        let total_rx_speed: f64 = snap.networks.iter().map(|n| n.rx_speed).sum();
        let total_tx_speed: f64 = snap.networks.iter().map(|n| n.tx_speed).sum();
        self.history.push_net(x, total_rx_speed, total_tx_speed);

        // Disk throughput (bytes/s) aggregated across disks, and the
        // 1-minute load average (UX8.3 data surface). Same bounded pattern.
        let total_disk_read: f64 = snap.disk_io.iter().map(|d| d.read_speed).sum();
        let total_disk_write: f64 = snap.disk_io.iter().map(|d| d.write_speed).sum();
        self.history.push_disk(x, total_disk_read, total_disk_write);
        self.history.push_load(x, snap.load_avg.one);

        // Per-process CPU samples (UX9.1): one per visible (sorted + capped)
        // process per tick. Bounded in the history struct itself, so memory
        // stays flat regardless of churn.
        for proc in &snap.processes {
            self.proc_cpu_history.push(proc.pid, proc.cpu_usage);
        }

        // Cache the sample for every widget and action in this frame.
        self.last_snapshot = Some(snap);

        // Let plugins tick
        let _ = self.with_plugin_manager_mut(|mgr, this| {
            mgr.tick_all(this);
        });
        self.refresh_plugin_widgets();
    }

    /// The current sample (one per tick). Widgets and process actions read
    /// this instead of resampling the system every frame.
    pub fn snapshot_cache(&self) -> Option<&SystemSnapshot> {
        self.last_snapshot.as_ref()
    }

    /// Full current snapshot. Prefer [`AppState::snapshot_cache`] in render
    /// paths; this forces a fresh system sample (used by plugin hosts).
    pub fn snapshot(&self) -> SystemSnapshot {
        self.last_snapshot
            .clone()
            .unwrap_or_else(|| self.provider.snapshot())
    }

    /// The process rows the UI shows: search filter + user sort, applied to
    /// one shared sample. Single source of truth for the processes widget and
    /// the Up/Down/Kill actions (selection is anchored by PID).
    ///
    /// The order honors [`AppState::process_sort_desc`]: descending puts the
    /// largest values first for CPU/Memory and Z-A for PID/Name, ascending
    /// the reverse. Descending is the default (classic boot order: CPU and
    /// Memory high-first).
    pub fn sorted_processes<'a>(&'a self, snap: &'a SystemSnapshot) -> Vec<&'a ProcessInfo> {
        let mut items: Vec<&ProcessInfo> = if self.search_query.is_empty() {
            snap.processes.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            snap.processes
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&q))
                .collect()
        };
        items.sort_by(|a, b| {
            let ord = match self.process_sort {
                ProcessSortBy::Cpu => a
                    .cpu_usage
                    .partial_cmp(&b.cpu_usage)
                    .unwrap_or(std::cmp::Ordering::Equal),
                ProcessSortBy::Memory => a.memory.cmp(&b.memory),
                ProcessSortBy::Pid => a.pid.cmp(&b.pid),
                ProcessSortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            };
            if self.process_sort_desc {
                ord.reverse()
            } else {
                ord
            }
        });
        items
    }

    pub fn next_theme(&mut self) {
        self.selected_theme_index = (self.selected_theme_index + 1) % self.themes.len();
        self.apply_theme();
    }

    pub fn previous_theme(&mut self) {
        if self.selected_theme_index == 0 {
            self.selected_theme_index = self.themes.len() - 1;
        } else {
            self.selected_theme_index -= 1;
        }
        self.apply_theme();
    }

    pub(crate) fn apply_theme(&mut self) {
        self.current_theme = self.themes[self.selected_theme_index].clone();
    }

    pub fn next_layout(&mut self) {
        if self.layout_defs.is_empty() {
            return;
        }
        self.layout_index = (self.layout_index + 1) % self.layout_defs.len();
        self.sync_layout_mode();
        self.full_screen_widget = FullScreenWidget::None;
    }

    pub fn current_layout_name(&self) -> &str {
        &self.layout_defs[self.layout_index].name
    }

    pub fn toggle_fullscreen(&mut self) {
        self.full_screen_widget = match self.full_screen_widget {
            FullScreenWidget::None => FullScreenWidget::Cpu,
            _ => FullScreenWidget::None,
        };
    }

    pub fn cycle_fullscreen_widget(&mut self) {
        self.full_screen_widget = self.full_screen_widget.next();
    }

    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Searching;
        self.search_query.clear();
    }

    pub fn search_push_char(&mut self, c: char) {
        self.search_query.push(c);
    }

    pub fn search_pop_char(&mut self) {
        self.search_query.pop();
    }

    pub fn end_search(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn process_select_next(&mut self) {
        self.move_process_selection(1);
    }

    pub fn process_select_prev(&mut self) {
        self.move_process_selection(-1);
    }

    fn move_process_selection(&mut self, dir: i32) {
        let Some(snap) = self.snapshot_cache() else {
            return;
        };
        let view = self.sorted_processes(snap);
        if view.is_empty() {
            return;
        }
        let n = view.len() as i32;
        let pos = self
            .process_selected_pid
            .and_then(|pid| view.iter().position(|p| p.pid == pid))
            .unwrap_or(0) as i32;
        let next = (pos + dir).rem_euclid(n);
        self.process_selected_pid = Some(view[next as usize].pid);
    }

    /// Cycle the process sort (the `s` key / "Sort" palette entry).
    ///
    /// Each press alternates between the two phases of the current column,
    /// so the full cycle covers every (column, direction) pair:
    ///
    /// 1. A descending column is toggled to ascending (first press = flip the
    ///    order on the current column, e.g. boot CPU% desc → CPU% asc);
    /// 2. an ascending column advances to the next column (CPU% → Mem →
    ///    PID → Name), which starts descending (next press = move on,
    ///    descending first — the boot default).
    ///
    /// The column order itself is unchanged from the pre-direction cycle.
    pub fn cycle_sort(&mut self) {
        if self.process_sort_desc {
            self.process_sort_desc = false;
        } else {
            self.process_sort = self.process_sort.next();
            self.process_sort_desc = true;
        }
        self.process_selected_pid = None;
    }
}
