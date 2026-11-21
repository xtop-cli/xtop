//! Windows terminal lifecycle for the TUI.
//!
//! crossterm handles the console mode, VT processing and raw input; the
//! shared enter/leave sequence and the panic hook live in
//! [`crate::tui::terminal_common`]. Future Windows-only terminal handling
//! (console mode quirks, Ctrl events, ConPTY differences) belongs in this
//! module.

use std::io;

use crate::tui::terminal_common as common;

pub(crate) fn init() -> io::Result<common::Tui> {
    // Build the terminal before touching the screen: a construction failure
    // must not leave the user in the alternate screen.
    let terminal = common::terminal()?;
    common::enter()?;
    Ok(terminal)
}

pub(crate) fn restore() -> io::Result<()> {
    common::leave()
}

pub(crate) fn install_panic_hook() {
    common::install_panic_hook(restore);
}
