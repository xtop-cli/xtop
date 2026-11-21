//! TUI frontend: run loop, terminal lifecycle, screen composition, overlays,
//! layout rendering and frame effects.
//!
//! The TUI consumes [`crate::core`] and never the CLI. Platform-specific
//! terminal code lives under `unix` and `windows`; shared terminal
//! primitives live in `terminal_common`.
//!
//! - [`terminal`]       platform terminal backend facade
//! - [`run()`]          interactive event/tick loop
//! - [`input`]          key mapping and mode dispatch
//! - [`screen`]         top-level composition (fullscreen, minimal, layout)
//! - [`layout`]         layout engine: rect split + widget pack resolution
//! - [`effects`]        optional frame-effect host (feature `effects`)
//! - [`overlay`]        kernel-owned UI chrome (help, command palette)

mod actions;
#[cfg(test)]
mod command_bar_tests;
pub mod effects;
pub mod input;
pub mod layout;
pub mod overlay;
mod palette;
pub mod run;
pub mod screen;
pub mod terminal;
pub(crate) mod terminal_common;
mod widget_state;

#[cfg(not(any(unix, windows)))]
mod fallback;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use run::run;
