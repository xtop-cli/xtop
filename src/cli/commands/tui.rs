//! TUI launch command.
//!
//! The CLI owns argument dispatch; the interactive application lives in
//! [`crate::tui`]. Keeping this boundary in one file makes the frontend
//! dependency explicit and lets the CLI grow headless commands without
//! linking them to the TUI modules.

/// Start the interactive TUI system monitor.
pub fn run_tui() -> anyhow::Result<()> {
    crate::tui::run()
}
