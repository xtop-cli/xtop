//! `xtop status`: one-shot snapshot or one focused section.

use crate::cli::help;

mod print;

pub(crate) use print::print_status;
use print::{print_status_section, status_section_value, StatusSection};

use super::monitor;

pub(crate) fn status_command(args: &[String]) -> anyhow::Result<()> {
    let rest = &args[2..];

    if help::wants_help(rest) {
        match rest.iter().find(|arg| !arg.starts_with('-')) {
            Some(raw) => {
                let section = parse_section(raw);
                show_section_help(section);
            }
            None => help::show(help::STATUS),
        }
        return Ok(());
    }

    let mut json = false;
    let mut section = StatusSection::Summary;
    let mut section_set = false;
    for arg in rest {
        match arg.as_str() {
            "--json" => json = true,
            other if other.starts_with('-') => {
                help::usage_error(help::STATUS, &format!("unknown option '{other}'"));
            }
            other => {
                if section_set {
                    help::usage_error(help::STATUS, "only one status section may be given");
                }
                section = parse_section(other);
                section_set = true;
            }
        }
    }

    let (snapshot, info) = monitor::sample();
    if json {
        let value = monitor::status_value(&snapshot, &info);
        monitor::print_json(&status_section_value(&value, section))
    } else {
        print_status_section(&snapshot, &info, section);
        Ok(())
    }
}

fn parse_section(raw: &str) -> StatusSection {
    StatusSection::parse(raw).unwrap_or_else(|| {
        help::usage_error(
            help::STATUS,
            &format!(
                "unknown status section '{raw}': use {}",
                StatusSection::names()
            ),
        )
    })
}

fn show_section_help(section: StatusSection) {
    if section == StatusSection::Summary {
        help::show(help::STATUS);
        return;
    }
    let key = section
        .json_key()
        .map(|key| format!("JSON key: {key}."))
        .unwrap_or_default();
    let text = format!(
        "xtop status {name} - {description}\n\nUsage:\n  xtop status {name} [--json]\n\n{key}\n\nOptions:\n  --json        Print a machine-readable JSON object\n  -h, --help    Show this help and exit\n\nRun 'xtop status --help' for the complete section list.",
        name = section.name(),
        description = section.description(),
        key = key,
    );
    help::show(&text);
}
