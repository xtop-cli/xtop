//! Windows TUI backend.
//!
//! Platform-specific terminal behavior lives here; the shared crossterm
//! sequence is composed from [`crate::tui::terminal_common`].

pub(crate) mod terminal;

pub(crate) use terminal::{init, install_panic_hook, restore};
