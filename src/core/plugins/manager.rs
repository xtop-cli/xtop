//! Plugin host: lifecycle, tick/key dispatch and capability routing.
use std::fmt::Debug;
use std::path::PathBuf;

use crate::core::state::AppState;
use xtop_plugin_api::SystemDataProvider;
use xtop_plugin_api::{Plugin, PluginCapability, PluginContext, PluginError, PluginWidget};

/// Manages the lifecycle of all loaded plugins.
///
/// Responsibilities:
/// - Loading and enabling plugins at startup
/// - Dispatching tick, key, and command events
/// - Collecting data providers and widgets from plugins
/// - Error isolation (a failing plugin does not crash xtop)
/// - Plugin persistence directory management
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_data_base: PathBuf,
}

impl Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("count", &self.plugins.len())
            .finish()
    }
}

impl PluginManager {
    /// Create a new manager with the base directory for plugin data.
    ///
    /// The base is the platform config dir (Linux `~/.config/xtop/plugins/`,
    /// macOS `~/Library/Application Support/xtop/plugins/`).
    pub fn new(plugin_data_base: PathBuf) -> Self {
        Self {
            plugins: Vec::new(),
            plugin_data_base,
        }
    }

    /// Register and enable a plugin.
    ///
    /// This calls `on_enable` on the plugin. If it fails, the plugin is not added
    /// and the error is logged. Only reachable when a plugin is compiled in.
    #[cfg_attr(not(feature = "plugin-samurai"), allow(dead_code))]
    pub fn register(
        &mut self,
        mut plugin: Box<dyn Plugin>,
        state: &mut AppState,
    ) -> Result<(), PluginError> {
        let id = plugin.manifest().id.clone();
        let data_dir = self.plugin_data_base.join(&id);
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            PluginError::Recoverable(format!("failed to create plugin data dir for {id}: {e}"))
        })?;

        let capabilities = plugin.manifest().capabilities.clone();
        let mut ctx = PluginContext::new(state, data_dir, capabilities);

        plugin.on_enable(&mut ctx)?;

        self.plugins.push(plugin);
        Ok(())
    }

    fn plugin_has_capability(plugin: &dyn Plugin, cap: &PluginCapability) -> bool {
        plugin.manifest().capabilities.contains(cap)
    }

    fn build_context<'a>(
        base: &std::path::Path,
        plugin: &dyn Plugin,
        state: &'a mut AppState,
    ) -> PluginContext<'a> {
        let id = plugin.manifest().id.clone();
        let capabilities = plugin.manifest().capabilities.clone();
        PluginContext::new(state, base.join(&id), capabilities)
    }

    /// Call `on_tick` on every enabled plugin.
    /// Errors are caught per-plugin so one failing plugin does not affect others.
    pub fn tick_all(&mut self, state: &mut AppState) {
        let base = self.plugin_data_base.clone();
        for plugin in &mut self.plugins {
            let id = plugin.manifest().id.clone();
            let mut ctx = Self::build_context(&base, &**plugin, state);
            if let Err(e) = plugin.on_tick(&mut ctx) {
                eprintln!("[plugin:{id}] tick error: {e}");
            }
        }
    }

    /// Dispatch a key event to all plugins.
    /// Returns `true` if any plugin consumed the event.
    pub fn handle_key(&mut self, state: &mut AppState, key: &str) -> bool {
        let base = self.plugin_data_base.clone();
        for plugin in &mut self.plugins {
            let id = plugin.manifest().id.clone();
            let mut ctx = Self::build_context(&base, &**plugin, state);
            match plugin.on_key(&mut ctx, key) {
                Ok(true) => return true,
                Err(e) => eprintln!("[plugin:{id}] key error: {e}"),
                _ => {}
            }
        }
        false
    }

    /// Collect all data providers from plugins for use in CompositeProvider.
    /// Only includes providers from plugins with `ReadSystemInfo` capability.
    pub fn collect_data_providers(&self) -> Vec<Box<dyn SystemDataProvider>> {
        let mut providers: Vec<Box<dyn SystemDataProvider>> = Vec::new();
        for plugin in &self.plugins {
            if Self::plugin_has_capability(&**plugin, &PluginCapability::ReadSystemInfo) {
                if let Some(provider) = plugin.data_provider() {
                    providers.push(provider);
                }
            }
        }
        providers
    }

    /// Collect all widget registrations from plugins.
    /// Only includes widgets from plugins with `RenderWidgets` capability.
    pub fn collect_widgets(&self) -> Vec<PluginWidget> {
        let mut widgets: Vec<PluginWidget> = Vec::new();
        for plugin in &self.plugins {
            if Self::plugin_has_capability(&**plugin, &PluginCapability::RenderWidgets) {
                if let Some(widget) = plugin.widget() {
                    widgets.push(widget);
                }
            }
        }
        widgets
    }

    /// Execute a command on the plugin identified by `plugin_id`.
    ///
    /// Returns the plugin's response string on success.
    pub fn execute(
        &mut self,
        state: &mut AppState,
        plugin_id: &str,
        action: &str,
        params: &str,
    ) -> Result<String, PluginError> {
        let base = self.plugin_data_base.clone();
        for plugin in &mut self.plugins {
            if plugin.manifest().id != plugin_id {
                continue;
            }
            let mut ctx = Self::build_context(&base, &**plugin, state);
            return plugin.execute(&mut ctx, action, params);
        }
        Err(PluginError::Recoverable(format!(
            "plugin not found: {plugin_id}"
        )))
    }

    /// Call `on_disable` on all plugins (e.g. on shutdown).
    pub fn disable_all(&mut self, state: &mut AppState) {
        let base = self.plugin_data_base.clone();
        for plugin in &mut self.plugins {
            let id = plugin.manifest().id.clone();
            let mut ctx = Self::build_context(&base, &**plugin, state);
            if let Err(e) = plugin.on_disable(&mut ctx) {
                eprintln!("[plugin:{id}] disable error: {e}");
            }
        }
    }
}
