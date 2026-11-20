//! xtop kernel library.
//!
//! The package ships one library and one binary. The library exposes the
//! layered architecture so the TUI, the CLI and future headless consumers
//! share one core:
//!
//! ```text
//! core/   reusable data, config, theme, providers, plugins, state
//! cli/    argument dispatch and user-facing commands (consumes core)
//! tui/    interactive rendering/input/terminal (consumes core)
//! ```
//!
//! Dependency direction is enforced by `scripts/audit.sh`: `core` never
//! imports `cli` or `tui`; `tui` never imports `cli`; the CLI reaches the TUI
//! only through `cli::commands::tui`.

pub mod cli;
pub mod core;
pub mod tui;
