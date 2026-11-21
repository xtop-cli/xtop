//! Fallback terminal lifecycle for non-Unix, non-Windows targets.

use std::io;

use crate::tui::terminal_common as common;

pub(crate) fn init() -> io::Result<common::Tui> {
    common::enter()?;
    common::terminal()
}

pub(crate) fn restore() -> io::Result<()> {
    common::leave()
}

pub(crate) fn install_panic_hook() {
    common::install_panic_hook(restore);
}
