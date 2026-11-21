//! Reusable kernel core: configuration, providers, state, themes and plugins.
//!
//! `core` is frontend-agnostic. It must not import `crate::cli` or
//! `crate::tui`; both frontends consume it through this module. Shared
//! bootstrap and asset-seeding helpers live here too, so a headless CLI and
//! the TUI assemble the same live state.

pub mod assets;
pub mod bootstrap;
pub mod config;
pub mod plugins;
pub mod providers;
pub mod state;
pub mod theme;
