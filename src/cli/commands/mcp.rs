//! MCP server command.
//!
//! Runs the `mcp` server provided by the `xtop-extension-mcp` extension over
//! a freshly initialized application state. The extension drives xtop
//! through the `xtop-extension-api` host contract (tick + plugin actions),
//! so the kernel only wires state into the extension context.
//!
//! Requires both the `plugin-samurai` (tools execute against it) and the
//! `mcp-extension` features.

#![cfg_attr(
    not(all(feature = "plugin-samurai", feature = "mcp-extension")),
    allow(dead_code, unused_imports)
)]

use crate::core::bootstrap::initialize_state;
use crate::core::config;

#[cfg(all(feature = "plugin-samurai", feature = "mcp-extension"))]
use xtop_extension_api::Extension;

/// Run the MCP server.
pub fn run_mcp_server() -> anyhow::Result<()> {
    #[cfg(all(feature = "plugin-samurai", feature = "mcp-extension"))]
    {
        let cfg_dir = config::config_dir();
        let mut state = initialize_state(&cfg_dir)?;

        let mut extension = xtop_extension_mcp::McpExtension::new();
        let mut ctx = xtop_extension_api::ExtensionContext::new(&mut state);
        extension
            .run_server("mcp", &mut ctx)
            .map_err(|e| anyhow::anyhow!("mcp server error: {e}"))
    }

    #[cfg(not(all(feature = "plugin-samurai", feature = "mcp-extension")))]
    {
        eprintln!("MCP server requires the 'plugin-samurai' and 'mcp-extension' features.");
        eprintln!("Rebuild with: cargo build --features plugin-samurai,mcp-extension");
        std::process::exit(1);
    }
}
