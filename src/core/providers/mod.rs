//! Providers area: data sources that feed the state.
//!
//! `composite` merges several providers; `sysinfo` implements the main
//! cross-platform provider with per-OS probes under its `platform` tree.

mod composite;

pub mod sysinfo;

pub use composite::*;
