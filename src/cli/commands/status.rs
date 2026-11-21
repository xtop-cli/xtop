//! `xtop status`: one-shot system snapshot.

use crate::cli::help;

use super::monitor;

pub(crate) fn status_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::STATUS);
        return Ok(());
    }
    let mut json = false;
    for arg in &args[2..] {
        match arg.as_str() {
            "--json" => json = true,
            other => help::usage_error(help::STATUS, &format!("unknown option '{other}'")),
        }
    }
    let (snapshot, info) = monitor::sample();
    if json {
        monitor::print_json(&monitor::status_value(&snapshot, &info))
    } else {
        monitor::print_status(&snapshot, &info);
        Ok(())
    }
}
