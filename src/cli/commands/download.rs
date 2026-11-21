//! Shared parsing and reporting for asset download commands.

use crate::cli::help;
use crate::core::assets::{AssetKind, Outcome, Report};

pub(crate) struct Request {
    pub all: bool,
    pub force: bool,
    pub query: Option<String>,
}

/// Parse `<name>` or `--all`, plus the optional `--force`.
pub(crate) fn parse(args: &[String], help_text: &str) -> Request {
    let mut all = false;
    let mut force = false;
    let mut query: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--all" => all = true,
            "--force" => force = true,
            value if value.starts_with('-') => {
                help::usage_error(help_text, &format!("unknown option '{value}'"));
            }
            value => {
                if query.is_some() {
                    help::usage_error(help_text, "only one asset name may be given");
                }
                query = Some(value.to_string());
            }
        }
    }
    if !all && query.is_none() {
        help::usage_error(help_text, "provide an asset name or --all");
    }
    if all && query.is_some() {
        help::usage_error(help_text, "choose either an asset name or --all");
    }
    Request { all, force, query }
}

/// Print one line per asset and return an error when any download failed.
pub(crate) fn print_reports(reports: &[Report]) -> anyhow::Result<()> {
    let mut failures = 0usize;
    for report in reports {
        let kind = match report.kind {
            AssetKind::Theme => "theme",
            AssetKind::Layout => "layout",
        };
        match &report.outcome {
            Outcome::Installed => {
                println!("Installed {kind} '{}' ({})", report.id, report.name);
            }
            Outcome::Updated => {
                println!("Updated {kind} '{}' ({})", report.id, report.name);
            }
            Outcome::Skipped => {
                println!(
                    "Skipped {kind} '{}' (already exists; use --force to replace)",
                    report.id
                );
            }
            Outcome::Failed(error) => {
                eprintln!("Failed {kind} '{}': {error}", report.id);
                failures += 1;
            }
        }
    }
    if failures > 0 {
        anyhow::bail!("{failures} asset download(s) failed");
    }
    Ok(())
}
