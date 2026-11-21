//! Developer tooling that edits the kernel source tree.
//!
//! These commands are intentionally separate from the user-facing commands:
//! they require a checkout, `cargo` and sometimes `git`, and they modify
//! `Cargo.toml` or source files.

pub mod plugins;
pub mod widget;
