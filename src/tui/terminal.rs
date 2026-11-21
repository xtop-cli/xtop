//! Terminal backend facade.
//!
//! Selects the platform backend (`unix`, `windows` or the fallback) and
//! keeps the TUI run loop free of `cfg` attributes. The platform modules own
//! their entry points; `crate::tui::terminal_common` holds the shared
//! crossterm/ratatui sequence.

use std::io;

#[cfg(not(any(unix, windows)))]
use crate::tui::fallback as platform;
#[cfg(unix)]
use crate::tui::unix as platform;
#[cfg(windows)]
use crate::tui::windows as platform;

pub(crate) use crate::tui::terminal_common::Tui;

pub(crate) fn init() -> io::Result<Tui> {
    platform::init()
}

pub(crate) fn restore() -> io::Result<()> {
    platform::restore()
}

pub(crate) fn install_panic_hook() {
    platform::install_panic_hook();
}
