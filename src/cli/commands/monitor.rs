//! Shared sampling and rendering for the headless monitoring commands.

use std::time::Duration;

use crate::core::providers::sysinfo::SysinfoProvider;
use crate::core::state::ProcessSortBy;
use serde_json::{json, Value};
use xtop_plugin_api::model::{ProcessInfo, SystemInfo, SystemSnapshot};
use xtop_plugin_api::SystemDataProvider;

const SAMPLE_SETTLE_MS: u64 = 200;

/// A live provider that has already taken the two samples CPU and rate
/// metrics need. Reuse it for watch mode so deltas are measured between
/// successive refreshes.
pub(crate) struct Monitor {
    provider: SysinfoProvider,
}

impl Monitor {
    pub(crate) fn new() -> Self {
        let mut provider = SysinfoProvider::new();
        provider.refresh_all();
        std::thread::sleep(Duration::from_millis(SAMPLE_SETTLE_MS));
        provider.refresh_all();
        Self { provider }
    }

    pub(crate) fn snapshot(&self) -> (SystemSnapshot, SystemInfo) {
        (self.provider.snapshot(), self.provider.system_info())
    }

    pub(crate) fn refresh(&mut self) {
        self.provider.refresh_all();
    }
}

/// One-shot sample for status and processes.
pub(crate) fn sample() -> (SystemSnapshot, SystemInfo) {
    Monitor::new().snapshot()
}

pub(crate) fn average_cpu(snapshot: &SystemSnapshot) -> f64 {
    if snapshot.cpus.is_empty() {
        0.0
    } else {
        let total: f64 = snapshot.cpus.iter().map(|cpu| cpu.usage).sum();
        total / snapshot.cpus.len() as f64
    }
}

pub(crate) fn status_value(snapshot: &SystemSnapshot, info: &SystemInfo) -> Value {
    let cores: Vec<Value> = snapshot
        .cpus
        .iter()
        .map(|cpu| {
            json!({
                "id": cpu.cpu_id,
                "name": cpu.name,
                "usage_percent": finite(cpu.usage),
                "frequency_mhz": cpu.frequency,
                "governor": cpu.governor,
                "temperature_c": cpu.temp_c.filter(|t| t.is_finite()),
            })
        })
        .collect();

    json!({
        "schema": 1,
        "kind": "status",
        "host": {
            "hostname": info.hostname,
            "os": info.os_version,
            "kernel": info.kernel,
            "desktop_env": info.desktop_env,
            "shell": info.shell,
            "cpu_model": info.cpu_model,
            "package_power_w": info.package_power_w.filter(|v| v.is_finite()),
        },
        "cpu": {
            "usage_percent": finite(average_cpu(snapshot)),
            "temperature_c": if snapshot.cpu_temp > 0.0 { Some(snapshot.cpu_temp) } else { None },
            "cores": cores,
        },
        "memory": memory_value(snapshot),
        "swap": swap_value(snapshot),
        "load": {
            "one": finite(snapshot.load_avg.one),
            "five": finite(snapshot.load_avg.five),
            "fifteen": finite(snapshot.load_avg.fifteen),
        },
        "uptime_seconds": snapshot.uptime,
        "disks": snapshot.disks.iter().map(|disk| json!({
            "mount_point": disk.mount_point,
            "file_system": disk.file_system,
            "mount_options": disk.mount_options,
            "total_bytes": disk.total_space,
            "used_bytes": disk.used_space,
            "available_bytes": disk.available_space,
            "percent": finite(disk.percent),
        })).collect::<Vec<_>>(),
        "disk_io": snapshot.disk_io.iter().map(|disk| json!({
            "name": disk.name,
            "read_bytes": disk.read_bytes,
            "write_bytes": disk.write_bytes,
            "read_bytes_per_second": finite(disk.read_speed),
            "write_bytes_per_second": finite(disk.write_speed),
        })).collect::<Vec<_>>(),
        "networks": snapshot.networks.iter().map(|net| json!({
            "name": net.name,
            "received_bytes": net.received,
            "transmitted_bytes": net.transmitted,
            "rx_bytes_per_second": finite(net.rx_speed),
            "tx_bytes_per_second": finite(net.tx_speed),
            "ip": net.ip,
        })).collect::<Vec<_>>(),
        "batteries": snapshot.batteries.iter().map(|battery| json!({
            "name": battery.name,
            "percent": finite(battery.percentage as f64),
            "state": battery.state,
            "time_to_full_seconds": battery.time_to_full,
            "time_to_empty_seconds": battery.time_to_empty,
            "health": finite(battery.health as f64),
            "cycle_count": battery.cycle_count,
        })).collect::<Vec<_>>(),
        "gpus": snapshot.gpus.iter().map(|gpu| json!({
            "name": gpu.name,
            "usage_percent": finite(gpu.usage),
            "temperature_c": finite(gpu.temperature as f64),
            "memory_total_bytes": gpu.memory_total,
            "memory_used_bytes": gpu.memory_used,
        })).collect::<Vec<_>>(),
    })
}

pub(crate) fn processes_value(
    snapshot: &SystemSnapshot,
    sort: ProcessSortBy,
    descending: bool,
    filter: &str,
    limit: usize,
) -> Value {
    let selected = select_processes(snapshot, sort, descending, filter, limit);
    json!({
        "schema": 1,
        "kind": "processes",
        "count": selected.len(),
        "processes": selected.iter().map(|process| process_value(process)).collect::<Vec<_>>(),
    })
}

pub(crate) fn select_processes<'a>(
    snapshot: &'a SystemSnapshot,
    sort: ProcessSortBy,
    descending: bool,
    filter: &str,
    limit: usize,
) -> Vec<&'a ProcessInfo> {
    let needle = filter.trim().to_ascii_lowercase();
    let mut processes: Vec<&ProcessInfo> = snapshot
        .processes
        .iter()
        .filter(|process| needle.is_empty() || process.name.to_ascii_lowercase().contains(&needle))
        .collect();
    processes.sort_by(|a, b| {
        let order = match sort {
            ProcessSortBy::Cpu => a
                .cpu_usage
                .partial_cmp(&b.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal),
            ProcessSortBy::Memory => a.memory.cmp(&b.memory),
            ProcessSortBy::Pid => a.pid.cmp(&b.pid),
            ProcessSortBy::Name => a
                .name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase()),
        };
        if descending {
            order.reverse()
        } else {
            order
        }
    });
    if limit > 0 {
        processes.truncate(limit);
    }
    processes
}

fn process_value(process: &ProcessInfo) -> Value {
    json!({
        "pid": process.pid,
        "name": process.name,
        "cpu_percent": finite(process.cpu_usage),
        "memory_bytes": process.memory,
        "user_id": process.user_id,
        "state": process.state,
        "cmd": process.cmd,
        "exe_path": process.exe_path,
        "parent_pid": process.parent_pid,
        "start_time_epoch": process.start_time,
        "run_time_seconds": process.run_time,
        "thread_count": process.thread_count,
        "open_files": process.open_files,
        "session_id": process.session_id,
    })
}

pub(crate) fn print_processes(
    snapshot: &SystemSnapshot,
    sort: ProcessSortBy,
    descending: bool,
    filter: &str,
    limit: usize,
) {
    println!(
        "{:<7} {:<24} {:>6} {:>12} {:<10} STATE",
        "PID", "NAME", "CPU%", "MEM", "USER"
    );
    for process in select_processes(snapshot, sort, descending, filter, limit) {
        println!(
            "{:<7} {:<24} {:>6.1} {:>12} {:<10} {}",
            process.pid,
            truncate(&process.name, 24),
            process.cpu_usage,
            format_bytes(process.memory),
            process.user_id.as_deref().unwrap_or("-"),
            process.state
        );
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(crate) fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub(crate) fn print_json(value: &Value) -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value)
            .map_err(|error| anyhow::anyhow!("failed to encode JSON: {error}"))?
    );
    Ok(())
}

fn memory_value(snapshot: &SystemSnapshot) -> Value {
    json!({
        "total_bytes": snapshot.memory.total,
        "used_bytes": snapshot.memory.used,
        "available_bytes": snapshot.memory.available,
        "free_bytes": snapshot.memory.free,
        "percent": finite(snapshot.memory.percent),
    })
}

fn swap_value(snapshot: &SystemSnapshot) -> Value {
    json!({
        "total_bytes": snapshot.swap.total,
        "used_bytes": snapshot.swap.used,
        "free_bytes": snapshot.swap.free,
        "percent": finite(snapshot.swap.percent),
    })
}

fn finite(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let mut out: String = value.chars().take(max.saturating_sub(1)).collect();
        out.push('~');
        out
    }
}
