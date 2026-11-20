//! Plugin host area: everything the kernel does to drive plugins.
//!
//! `manager` is the host side of the plugin lifecycle; `host` bridges the
//! kernel `AppState` to the contract view `xtop_plugin_api::HostState`;
//! `extension_host` does the same for extensions.

mod extension_host;
mod host;
mod manager;

pub use manager::*;
