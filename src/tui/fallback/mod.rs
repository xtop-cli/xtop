//! TUI backend for platforms that are neither Unix-like nor Windows.
//!
//! Keeps the same terminal contract as the other backends so the kernel
//! still compiles where crossterm provides a console backend.

pub(crate) mod terminal;

pub(crate) use terminal::{init, install_panic_hook, restore};
