//! CLI commands: dispatch targets for the `xtop` binary.
//!
//! Commands consume [`crate::core`]; only [`tui`] crosses into the TUI
//! frontend. Shared assembly and asset helpers live in `core::bootstrap` and
//! `core::assets`.

pub mod config;
mod download;
pub mod layout;
pub mod mcp;
pub mod plugins;
pub mod theme;
pub mod tui;
pub mod widget;
