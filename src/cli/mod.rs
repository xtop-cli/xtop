//! CLI frontend: argument dispatch and user-facing commands.
//!
//! The CLI consumes [`crate::core`]. The only deliberate edge into the TUI
//! is `commands::tui`, which launches the interactive app; everything else
//! here is headless or source-management tooling.

mod commands;
mod dev;
pub(crate) mod help;

use crate::core::assets::ensure_default_assets;

/// Run the CLI. No arguments (or an unrecognized first argument) starts the
/// TUI, preserving the original `xtop` entry behavior.
pub fn run() -> anyhow::Result<()> {
    reset_sigpipe();
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "mcp" => {
                if help::wants_help(&args[2..]) {
                    help::show(help::MCP);
                    return Ok(());
                }
                if args.len() > 2 {
                    help::usage_error(help::MCP, "Usage: xtop mcp");
                }
                ensure_default_assets();
                return commands::mcp::run_mcp_server();
            }
            "plugin" => {
                return dev::plugins::plugin_command(&args);
            }
            "widget" => {
                return dev::widget::widget_command(&args);
            }
            "status" => {
                return commands::status::status_command(&args);
            }
            "processes" => {
                return commands::processes::processes_command(&args);
            }
            "watch" => {
                return commands::watch::watch_command(&args);
            }
            "info" => {
                return commands::system::info_command(&args);
            }
            "doctor" => {
                return commands::system::doctor_command(&args);
            }
            "completions" => {
                return commands::completions::completions_command(&args);
            }
            "theme" => {
                return commands::theme::theme_command(&args);
            }
            "layout" => {
                return commands::layout::layout_command(&args);
            }
            "config" => {
                return commands::config::config_command(&args);
            }
            "--change-theme" | "--ct" => {
                if help::wants_help(&args[2..]) {
                    help::show(help::CHANGE_THEME);
                    return Ok(());
                }
                let Some(name) = args.get(2) else {
                    help::usage_error(help::CHANGE_THEME, "Usage: xtop --change-theme <theme>");
                };
                ensure_default_assets();
                return commands::theme::set_theme(name);
            }
            "--gen-config" => {
                if help::wants_help(&args[2..]) {
                    help::show(help::GEN_CONFIG);
                    return Ok(());
                }
                if args.len() > 2 {
                    help::usage_error(help::GEN_CONFIG, "Usage: xtop --gen-config");
                }
                return commands::config::generate_base_config();
            }
            "--version" | "-V" => {
                println!("xtop {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                help::show(help::GLOBAL);
                return Ok(());
            }
            _ => {}
        }
    }

    ensure_default_assets();
    commands::tui::run_tui()
}

#[cfg(unix)]
fn reset_sigpipe() {
    // Restore the default SIGPIPE behavior so `xtop status | head` exits
    // quietly instead of panicking on a closed stdout.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}
