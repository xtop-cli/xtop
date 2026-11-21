//! Composite data provider: kernel provider plus plugin providers.
use xtop_plugin_api::model::*;
use xtop_plugin_api::SystemDataProvider;

/// A `SystemDataProvider` that composes a primary provider with plugin-provided extras.
///
/// The primary provider (usually `SysinfoProvider`) handles `refresh_all()` and `snapshot()`.
/// Extra providers (from plugins) override specific methods like `gpu_info()`, `batteries()`,
/// `disk_io()`, or `system_info()`.
///
/// This allows plugins to inject data without modifying the primary provider.
pub struct CompositeProvider {
    primary: Box<dyn SystemDataProvider>,
    extras: Vec<Box<dyn SystemDataProvider>>,
}

impl CompositeProvider {
    pub fn new(primary: Box<dyn SystemDataProvider>) -> Self {
        Self {
            primary,
            extras: Vec::new(),
        }
    }

    /// Get the first non-empty result from extras for a method,
    /// falling back to a fallback closure on the primary.
    fn first_non_empty<T: Clone>(
        &self,
        primary_fn: impl FnOnce() -> Vec<T>,
        extra_fn: impl Fn(&dyn SystemDataProvider) -> Vec<T>,
    ) -> Vec<T> {
        let primary_val = primary_fn();
        if !primary_val.is_empty() {
            return primary_val;
        }
        for extra in &self.extras {
            let val = extra_fn(extra.as_ref());
            if !val.is_empty() {
                return val;
            }
        }
        primary_val
    }
}

impl SystemDataProvider for CompositeProvider {
    fn refresh_all(&mut self) {
        self.primary.refresh_all();
        for extra in &mut self.extras {
            extra.refresh_all();
        }
    }

    fn snapshot(&self) -> SystemSnapshot {
        self.primary.snapshot()
    }

    fn disk_io(&self) -> Vec<DiskIOInfo> {
        self.first_non_empty(|| self.primary.disk_io(), |e| e.disk_io())
    }

    fn batteries(&self) -> Vec<BatteryInfo> {
        self.first_non_empty(|| self.primary.batteries(), |e| e.batteries())
    }

    fn gpu_info(&self) -> Vec<GpuInfo> {
        self.first_non_empty(|| self.primary.gpu_info(), |e| e.gpu_info())
    }

    fn system_info(&self) -> SystemInfo {
        let primary = self.primary.system_info();
        if !primary.hostname.is_empty() {
            return primary;
        }
        for extra in &self.extras {
            let val = extra.system_info();
            if !val.hostname.is_empty() {
                return val;
            }
        }
        primary
    }

    fn kill_process(&self, pid: u32) -> bool {
        if self.primary.kill_process(pid) {
            return true;
        }
        for extra in &self.extras {
            if extra.kill_process(pid) {
                return true;
            }
        }
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn add_extras(&mut self, extras: Vec<Box<dyn SystemDataProvider>>) {
        self.extras = extras;
    }
}
