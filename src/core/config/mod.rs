//! Configuration area: file persistence, schema, keybindings and platform
//! dirs.

mod io;
pub mod keybinding;
mod persist;
mod platform;
mod schema;

pub use io::{config_path, load_config, save_config};
pub use persist::{persisted_layout, persisted_selection, persisted_theme, PersistedSelection};
pub use platform::config_dir;
pub use schema::*;
