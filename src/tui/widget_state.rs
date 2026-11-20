//! Kernel implementation of the widget renderer contract
//! ([`xtop_widget_api::WidgetState`]).
//!
//! This is the single door widget packs cross: they render against this view
//! and never touch kernel types.

use crate::core::state::app::AppState;
use crate::core::state::view::{FullScreenWidget, InputMode};
use xtop_plugin_api::model::{ProcessInfo, SystemSnapshot};

impl xtop_widget_api::WidgetState for AppState {
    fn snapshot(&self) -> Option<&SystemSnapshot> {
        self.snapshot_cache()
    }

    fn theme_name(&self) -> &str {
        &self.current_theme.name
    }

    fn theme_fg(&self) -> &[u8; 3] {
        self.current_theme.fg()
    }

    fn theme_bg(&self) -> &[u8; 3] {
        self.current_theme.bg()
    }

    fn theme_palette(&self) -> &[[u8; 3]; 16] {
        &self.current_theme.palette
    }

    fn alerts(&self) -> xtop_plugin_api::AlertThresholds {
        self.alerts.clone()
    }

    fn charset(&self, widget: &str) -> xtop_widget_api::ChartCharset {
        self.style.charset_for(widget)
    }

    fn borders(&self, widget: &str) -> xtop_widget_api::WidgetBorders {
        self.style.borders_for(widget)
    }

    fn cpu_history(&self) -> &[std::collections::VecDeque<(f64, f64)>] {
        &self.history.cpu
    }

    fn mem_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.mem
    }

    fn net_rx_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.net_rx
    }

    fn net_tx_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.net_tx
    }

    fn disk_read_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.disk_read
    }

    fn disk_write_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.disk_write
    }

    fn load_history(&self) -> &std::collections::VecDeque<(f64, f64)> {
        &self.history.load
    }

    fn search_query(&self) -> &str {
        &self.search_query
    }

    fn process_selected_pid(&self) -> Option<u32> {
        self.process_selected_pid
    }

    fn process_sort_label(&self) -> &str {
        self.process_sort.label()
    }

    fn process_sort_desc(&self) -> bool {
        // Descending default (CPU%/Mem high-first); the sort key toggles the
        // direction of the active column before advancing (app.rs cycle_sort).
        self.process_sort_desc
    }

    fn layout_name(&self) -> &str {
        self.current_layout_name()
    }

    fn is_searching(&self) -> bool {
        self.input_mode == InputMode::Searching
    }

    fn fullscreen_label(&self) -> Option<&str> {
        if self.full_screen_widget == FullScreenWidget::None {
            None
        } else {
            Some(self.full_screen_widget.label())
        }
    }

    fn sys_info(&self) -> xtop_plugin_api::SystemInfo {
        self.sys_info.clone()
    }

    fn process_view(&self) -> Vec<&ProcessInfo> {
        let Some(snap) = self.snapshot_cache() else {
            return Vec::new();
        };
        self.sorted_processes(snap)
    }

    fn uid_to_name(&self, uid: u32) -> Option<String> {
        self.users.name_for(uid).map(str::to_string)
    }

    fn process_cpu_history(&self, pid: u32) -> Vec<f64> {
        self.proc_cpu_history.history(pid)
    }

    fn logical_core_count(&self) -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    fn widget_options(&self) -> Option<&serde_json::Value> {
        // Set by the render engine around each pack-widget render call (the
        // active layout node's `options`); None outside renders, for widget
        // instances without options, and for plugin widget renders.
        self.active_widget_options.as_ref()
    }
}
