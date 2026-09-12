//! xtop CLI entry point.
//!
//! The binary is a thin dispatcher; the app is organized in areas:
//! `commands`, `config`, `plugins`, `providers`, `state`, `theme` and `ui`.
//! Layouts and widget packs live in their own repos and are consumed as
//! crates.

mod commands;
mod config;
mod plugins;
mod providers;
mod state;
mod theme;
mod ui;

use commands::{ensure_default_assets, mcp, run};

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  xtop                        Start the TUI system monitor");
    eprintln!("  xtop mcp                    Start MCP server (stdio transport) for AI agents");
    eprintln!("  xtop plugin list            List installed plugins");
    eprintln!("  xtop plugin install <name>  Install a plugin from github.com/xtop-cli/plugins");
    eprintln!("  xtop plugin install <url>   Install a plugin from a git URL");
    eprintln!("  xtop plugin scaffold <name> Create a new plugin crate");
    eprintln!("  xtop widget list            List installed widget packs");
    eprintln!("  xtop widget install <name>  Install a pack from github.com/xtop-cli/widgets");
    eprintln!("  xtop widget install <url|path>  Install a widget pack from a git URL or path");
    eprintln!("  xtop widget scaffold <name> Create a new single-widget pack crate");
    eprintln!("  xtop layout check <file>    Validate a layout JSONC file");
    eprintln!("  xtop layout install <name>  Install a layout from github.com/xtop-cli/layouts");
    eprintln!(
        "  xtop --ct <theme>           Change the active theme (persists; live instances follow)"
    );
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // CLI subcommands.
    if args.len() > 1 {
        match args[1].as_str() {
            "mcp" => {
                ensure_default_assets();
                return mcp::run_mcp_server();
            }
            "plugin" => {
                return commands::plugins::plugin_command(&args);
            }
            "widget" => {
                return commands::widget::widget_command(&args);
            }
            "layout" => {
                return commands::layout::layout_command(&args);
            }
            "--ct" => {
                ensure_default_assets();
                let Some(name) = args.get(2) else {
                    eprintln!("Usage: xtop --ct <theme>");
                    std::process::exit(2);
                };
                return commands::theme::cmd_change_theme(name);
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            _ => {}
        }
    }

    ensure_default_assets();
    run::run()
}
