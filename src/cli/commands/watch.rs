//! `xtop watch`: periodic system snapshots for terminals and scripts.

use std::time::Duration;

use crate::cli::help;

use super::monitor;

pub(crate) fn watch_command(args: &[String]) -> anyhow::Result<()> {
    if help::wants_help(&args[2..]) {
        help::show(help::WATCH);
        return Ok(());
    }

    let mut json_lines = false;
    let mut interval_ms = 1_000u64;
    let mut count = 0usize;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--json" | "--jsonl" => json_lines = true,
            "--interval" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::WATCH, "--interval needs milliseconds");
                };
                interval_ms = value
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("invalid --interval '{value}'"))?
                    .max(100);
            }
            "--count" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    help::usage_error(help::WATCH, "--count needs a number");
                };
                count = value
                    .parse::<usize>()
                    .map_err(|_| anyhow::anyhow!("invalid --count '{value}'"))?;
            }
            other => help::usage_error(help::WATCH, &format!("unknown option '{other}'")),
        }
        index += 1;
    }

    let mut monitor = monitor::Monitor::new();
    let mut sample = 0usize;
    loop {
        let (snapshot, info) = monitor.snapshot();
        sample += 1;
        if json_lines {
            let value = monitor::status_value(&snapshot, &info);
            println!(
                "{}",
                serde_json::to_string(&value)
                    .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
            );
        } else {
            if sample > 1 {
                println!();
            }
            println!("sample {sample}");
            super::status::print_status(&snapshot, &info);
        }
        if count > 0 && sample >= count {
            break;
        }
        std::thread::sleep(Duration::from_millis(interval_ms));
        monitor.refresh();
    }
    Ok(())
}
