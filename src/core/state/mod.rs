//! State area: live application state and metrics history.
//!
//! `app` holds the full `AppState`; `history` keeps historical snapshots for
//! charts; `view` groups the view-control types shared with the TUI
//! (fullscreen, input mode, palette). The persisted config schema lives under
//! `core::config`.

pub mod app;
#[cfg(test)]
mod app_tests;
pub mod history;
mod persist;
mod proc_history;
mod users;
pub mod view;

pub use app::*;
pub use persist::*;
pub use view::*;
