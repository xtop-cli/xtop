//! Kernel-side implementation of the extension host contract
//! (`xtop-extension-api`).
//!
//! Extensions drive the app by ticking it and executing named actions on
//! hosted plugins; both are delegated to the live [`AppState`].

use xtop_extension_api::{ExtensionError, ExtensionHost};

use crate::core::state::AppState;

impl ExtensionHost for AppState {
    fn tick(&mut self) {
        AppState::on_tick(self);
    }

    fn execute_plugin(
        &mut self,
        plugin_id: &str,
        action: &str,
        params: &str,
    ) -> Result<String, ExtensionError> {
        self.with_plugin_manager_mut(|mgr, this| {
            mgr.execute(this, plugin_id, action, params)
                .map_err(map_plugin_error)
        })
        .unwrap_or(Err(ExtensionError::Recoverable(
            "plugin manager not initialized".to_string(),
        )))
    }
}

fn map_plugin_error(e: xtop_plugin_api::PluginError) -> ExtensionError {
    use xtop_plugin_api::PluginError;
    match e {
        PluginError::Recoverable(msg) => ExtensionError::Recoverable(msg),
        PluginError::Fatal(msg) => ExtensionError::Fatal(msg),
        PluginError::UnknownAction(action) => {
            ExtensionError::Recoverable(format!("unknown action: {action}"))
        }
    }
}
