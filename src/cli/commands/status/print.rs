//! Focused `xtop status` sections and their human/JSON renderers.

use super::super::monitor::{average_cpu, format_bytes, format_uptime};
use serde_json::Value;
use xtop_plugin_api::model::{SystemInfo, SystemSnapshot};

/// One section of the `xtop status` snapshot. `Summary` keeps the full
/// historical output; every other variant is a focused quick query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatusSection {
    Summary,
    Host,
    Cpu,
    Memory,
    Swap,
    Load,
    Uptime,
    Disks,
    DiskIo,
    Network,
    Battery,
    Gpu,
}

impl StatusSection {
    pub(crate) const ALL: [Self; 12] = [
        Self::Summary,
        Self::Host,
        Self::Cpu,
        Self::Memory,
        Self::Swap,
        Self::Load,
        Self::Uptime,
        Self::Disks,
        Self::DiskIo,
        Self::Network,
        Self::Battery,
        Self::Gpu,
    ];

    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "summary" | "all" => Some(Self::Summary),
            "host" => Some(Self::Host),
            "cpu" => Some(Self::Cpu),
            "memory" | "mem" | "ram" => Some(Self::Memory),
            "swap" => Some(Self::Swap),
            "load" => Some(Self::Load),
            "uptime" => Some(Self::Uptime),
            "disks" | "disk" => Some(Self::Disks),
            "disk-io" | "disk_io" | "io" => Some(Self::DiskIo),
            "network" | "net" => Some(Self::Network),
            "battery" | "batteries" => Some(Self::Battery),
            "gpu" | "gpus" => Some(Self::Gpu),
            _ => None,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Host => "host",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Swap => "swap",
            Self::Load => "load",
            Self::Uptime => "uptime",
            Self::Disks => "disks",
            Self::DiskIo => "disk-io",
            Self::Network => "network",
            Self::Battery => "battery",
            Self::Gpu => "gpu",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Summary => "Complete snapshot (default).",
            Self::Host => "Hostname, OS, kernel, desktop, shell and CPU model.",
            Self::Cpu => "Aggregate CPU usage and temperature, plus per-core details.",
            Self::Memory => "RAM total, used, available, free and used percentage.",
            Self::Swap => "Swap total, used, free and used percentage.",
            Self::Load => "One, five and fifteen minute load averages.",
            Self::Uptime => "System uptime.",
            Self::Disks => "Mount points, filesystems, used and available space.",
            Self::DiskIo => "Per-device read/write totals and current rates.",
            Self::Network => "Interfaces with current RX/TX rates.",
            Self::Battery => "Batteries with charge percentage and state.",
            Self::Gpu => "GPUs with usage, temperature and memory.",
        }
    }

    pub(crate) fn json_key(self) -> Option<&'static str> {
        match self {
            Self::Summary => None,
            Self::Host => Some("host"),
            Self::Cpu => Some("cpu"),
            Self::Memory => Some("memory"),
            Self::Swap => Some("swap"),
            Self::Load => Some("load"),
            Self::Uptime => Some("uptime_seconds"),
            Self::Disks => Some("disks"),
            Self::DiskIo => Some("disk_io"),
            Self::Network => Some("networks"),
            Self::Battery => Some("batteries"),
            Self::Gpu => Some("gpus"),
        }
    }

    pub(crate) fn names() -> String {
        Self::ALL
            .iter()
            .map(|section| section.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(crate) fn status_section_value(value: &Value, section: StatusSection) -> Value {
    let Some(key) = section.json_key() else {
        return value.clone();
    };
    let mut object = serde_json::Map::new();
    for key in ["schema", "kind"] {
        if let Some(value) = value.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
    if let Some(value) = value.get(key) {
        object.insert(key.to_string(), value.clone());
    }
    Value::Object(object)
}

pub(crate) fn print_status(snapshot: &SystemSnapshot, info: &SystemInfo) {
    print_host(info);
    print_cpu_model(info);
    print_cpu(snapshot);
    print_memory(snapshot);
    print_swap(snapshot);
    print_load(snapshot);
    print_uptime(snapshot);
    print_disks(snapshot);
    print_disk_io(snapshot);
    print_networks(snapshot);
    print_batteries(snapshot);
    print_gpus(snapshot);
}

pub(crate) fn print_status_section(
    snapshot: &SystemSnapshot,
    info: &SystemInfo,
    section: StatusSection,
) {
    match section {
        StatusSection::Summary => print_status(snapshot, info),
        StatusSection::Host => print_host_section(info),
        StatusSection::Cpu => {
            print_cpu_model(info);
            print_cpu(snapshot);
            print_cpu_cores(snapshot);
        }
        StatusSection::Memory => print_memory_section(snapshot),
        StatusSection::Swap => print_swap(snapshot),
        StatusSection::Load => print_load(snapshot),
        StatusSection::Uptime => print_uptime(snapshot),
        StatusSection::Disks => {
            if snapshot.disks.is_empty() {
                println!("disks: none");
            } else {
                print_disks(snapshot);
            }
        }
        StatusSection::DiskIo => {
            if snapshot.disk_io.is_empty() {
                println!("disk_io: none");
            } else {
                print_disk_io(snapshot);
            }
        }
        StatusSection::Network => {
            if snapshot.networks.is_empty() {
                println!("network: none");
            } else {
                print_networks(snapshot);
            }
        }
        StatusSection::Battery => {
            if snapshot.batteries.is_empty() {
                println!("battery: none");
            } else {
                print_batteries(snapshot);
            }
        }
        StatusSection::Gpu => {
            if snapshot.gpus.is_empty() {
                println!("gpu: none");
            } else {
                print_gpus(snapshot);
            }
        }
    }
}

fn print_host(info: &SystemInfo) {
    println!("host: {}", empty_dash(&info.hostname));
    println!(
        "os: {} ({})",
        empty_dash(&info.os_version),
        empty_dash(&info.kernel)
    );
}

fn print_host_section(info: &SystemInfo) {
    print_host(info);
    print_cpu_model(info);
    if !info.desktop_env.is_empty() {
        println!("desktop: {}", info.desktop_env);
    }
    if !info.shell.is_empty() {
        println!("shell: {}", info.shell);
    }
    if let Some(power) = info.package_power_w.filter(|value| value.is_finite()) {
        println!("package power: {power:.1} W");
    }
}

fn print_cpu_model(info: &SystemInfo) {
    if let Some(model) = info.cpu_model.as_deref() {
        println!("cpu model: {model}");
    }
}

fn print_cpu(snapshot: &SystemSnapshot) {
    let temp = if snapshot.cpu_temp > 0.0 {
        format!(", {:.1} C", snapshot.cpu_temp)
    } else {
        String::new()
    };
    println!("cpu: {:.1}%{}", average_cpu(snapshot), temp);
}

fn print_cpu_cores(snapshot: &SystemSnapshot) {
    if snapshot.cpus.is_empty() {
        return;
    }
    println!("cores:");
    for cpu in &snapshot.cpus {
        let temp = cpu
            .temp_c
            .filter(|value| value.is_finite())
            .map(|value| format!(" {value:.1} C"))
            .unwrap_or_default();
        let governor = if cpu.governor.is_empty() {
            String::new()
        } else {
            format!(" {}", cpu.governor)
        };
        println!(
            "  {:<8} {:>6.1}%  {:>5} MHz{}{}",
            cpu.name, cpu.usage, cpu.frequency, temp, governor
        );
    }
}

fn print_memory(snapshot: &SystemSnapshot) {
    println!(
        "memory: {} / {} ({:.1}%)",
        format_bytes(snapshot.memory.used),
        format_bytes(snapshot.memory.total),
        snapshot.memory.percent
    );
}

fn print_memory_section(snapshot: &SystemSnapshot) {
    println!(
        "memory: {} / {} ({:.1}%)  available {}  free {}",
        format_bytes(snapshot.memory.used),
        format_bytes(snapshot.memory.total),
        snapshot.memory.percent,
        format_bytes(snapshot.memory.available),
        format_bytes(snapshot.memory.free)
    );
}

fn print_swap(snapshot: &SystemSnapshot) {
    println!(
        "swap: {} / {} ({:.1}%)",
        format_bytes(snapshot.swap.used),
        format_bytes(snapshot.swap.total),
        snapshot.swap.percent
    );
}

fn print_load(snapshot: &SystemSnapshot) {
    println!(
        "load: {:.2} {:.2} {:.2}",
        snapshot.load_avg.one, snapshot.load_avg.five, snapshot.load_avg.fifteen
    );
}

fn print_uptime(snapshot: &SystemSnapshot) {
    println!("uptime: {}", format_uptime(snapshot.uptime));
}

fn print_disks(snapshot: &SystemSnapshot) {
    if snapshot.disks.is_empty() {
        return;
    }
    println!("disks:");
    for disk in &snapshot.disks {
        println!(
            "  {:<20} {} / {} ({:.1}%) {}",
            disk.mount_point,
            format_bytes(disk.used_space),
            format_bytes(disk.total_space),
            disk.percent,
            disk.file_system
        );
    }
}

fn print_disk_io(snapshot: &SystemSnapshot) {
    if snapshot.disk_io.is_empty() {
        return;
    }
    println!("disk_io:");
    for disk in &snapshot.disk_io {
        println!(
            "  {:<20} read {}/s write {}/s (total {} / {})",
            disk.name,
            format_bytes(disk.read_speed as u64),
            format_bytes(disk.write_speed as u64),
            format_bytes(disk.read_bytes),
            format_bytes(disk.write_bytes)
        );
    }
}

fn print_networks(snapshot: &SystemSnapshot) {
    if snapshot.networks.is_empty() {
        return;
    }
    println!("network:");
    for net in &snapshot.networks {
        println!(
            "  {:<16} rx {}/s  tx {}/s",
            net.name,
            format_bytes(net.rx_speed as u64),
            format_bytes(net.tx_speed as u64)
        );
    }
}

fn print_batteries(snapshot: &SystemSnapshot) {
    for battery in &snapshot.batteries {
        println!(
            "battery: {} {:.1}% ({})",
            battery.name, battery.percentage, battery.state
        );
    }
}

fn print_gpus(snapshot: &SystemSnapshot) {
    for gpu in &snapshot.gpus {
        println!(
            "gpu: {} {:.1}% {:.1} C {} / {}",
            gpu.name,
            gpu.usage,
            gpu.temperature,
            format_bytes(gpu.memory_used),
            format_bytes(gpu.memory_total)
        );
    }
}

fn empty_dash(value: &str) -> &str {
    if value.is_empty() {
        "-"
    } else {
        value
    }
}
