//! Cross-platform terminal primitives shared by the TUI backends.
//!
//! The `unix` and `windows` backends own the platform entry points
//! ([`crate::tui::terminal`] dispatches to them); this module only holds the
//! crossterm/ratatui sequence both backends compose.

use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::panic;

pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Enter the alternate screen, capture mouse/paste and enable raw mode.
pub(crate) fn enter() -> io::Result<()> {
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    enable_raw_mode()
}

/// Restore the terminal to its pre-TUI state.
pub(crate) fn leave() -> io::Result<()> {
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    disable_raw_mode()
}

/// Build the ratatui terminal on top of stdout.
pub(crate) fn terminal() -> io::Result<Tui> {
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

/// Install a panic hook that restores the terminal before the original hook.
pub(crate) fn install_panic_hook(restore: fn() -> io::Result<()>) {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));
}
