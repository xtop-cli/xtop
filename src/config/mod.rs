//! Configuration area: file persistence, schema, keybindings and platform
//! dirs.

mod io;
pub mod keybinding;
mod platform;
mod schema;

pub use io::{config_path, load_config, save_config};
pub use platform::config_dir;
pub use schema::*;
