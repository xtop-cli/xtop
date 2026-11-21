//! Kernel-side implementation of the plugin host contract (`xtop-plugin-api`).
//!
//! The live [`AppState`] is what plugins see through [`HostState`], so plugin
//! code never depends on kernel types.

use crate::core::state::AppState;
use xtop_plugin_api::{AlertThresholds, HostState, RuntimeConfig, SystemInfo, SystemSnapshot};

impl HostState for AppState {
    fn snapshot(&self) -> SystemSnapshot {
        AppState::snapshot(self)
    }

    fn system_info(&self) -> SystemInfo {
        self.sys_info.clone()
    }

    fn kill_process(&mut self, pid: u32) -> bool {
        AppState::kill_process_by_pid(self, pid)
    }

    fn set_alert_thresholds(&mut self, cpu: f64, mem: f64, disk: f64) {
        AppState::set_alert_thresholds(self, cpu, mem, disk);
    }

    fn alerts(&self) -> AlertThresholds {
        self.alerts.clone()
    }

    fn config(&self) -> RuntimeConfig {
        RuntimeConfig {
            theme: self.current_theme.name.clone(),
            layout: self.current_layout_name().to_string(),
            interval_ms: self.update_interval_ms,
            hostname: self.sys_info.hostname.clone(),
        }
    }

    fn set_theme_by_name(&mut self, name: &str) -> bool {
        AppState::set_theme_by_name(self, name)
    }

    fn set_layout_by_name(&mut self, name: &str) -> bool {
        AppState::set_layout_by_name(self, name)
    }

    fn set_update_interval_ms(&mut self, ms: u64) {
        self.update_interval_ms = ms;
    }
}
