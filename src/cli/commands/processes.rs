//! `xtop processes`: one-shot process list with optional JSON output.

use crate::cli::help;
use crate::core::state::ProcessSortBy;

use super::monitor;

pub(crate) fn processes_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::PROCESSES);
        return Ok(());
    }

    let mut json = false;
    let mut sort = ProcessSortBy::Cpu;
    let mut descending = true;
    let mut limit = 20usize;
    let mut filter = String::new();
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--asc" => descending = false,
            "--desc" => descending = true,
            "--sort" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::PROCESSES, "--sort needs cpu, memory, pid or name");
                };
                sort = parse_sort(value, help::PROCESSES);
            }
            "--limit" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::PROCESSES, "--limit needs a number");
                };
                limit = value.parse().map_err(|_| {
                    anyhow::anyhow!("invalid --limit '{value}': expected a non-negative number")
                })?;
            }
            "--filter" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::PROCESSES, "--filter needs text");
                };
                filter = value.clone();
            }
            other => help::usage_error(help::PROCESSES, &format!("unknown option '{other}'")),
        }
        index += 1;
    }

    let (snapshot, _info) = monitor::sample();
    if json {
        monitor::print_json(&monitor::processes_value(
            &snapshot, sort, descending, &filter, limit,
        ))
    } else {
        monitor::print_processes(&snapshot, sort, descending, &filter, limit);
        Ok(())
    }
}

pub(crate) fn parse_sort(value: &str, help_text: &str) -> ProcessSortBy {
    match value.to_ascii_lowercase().as_str() {
        "cpu" => ProcessSortBy::Cpu,
        "memory" | "mem" => ProcessSortBy::Memory,
        "pid" => ProcessSortBy::Pid,
        "name" => ProcessSortBy::Name,
        _ => help::usage_error(help_text, "sort must be cpu, memory, pid or name"),
    }
}
