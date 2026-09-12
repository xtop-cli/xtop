//! CLI commands: interactive run, MCP server, plugin and widget-pack
//! management.
//!
//! Shared assembly and asset helpers live under [`share`].

pub mod layout;
pub mod mcp;
pub mod plugins;
pub mod run;
pub(crate) mod share;
pub mod theme;
pub mod widget;

pub(crate) use share::*;
