//! Fallback probes for any other Unix-like target (BSD, etc.).
//!
//! Keeps the provider compiling everywhere while only the sysinfo crate data
//! is available on those platforms.

use std::collections::HashMap;
use xtop_plugin_api::model::{BatteryInfo, GpuInfo};

pub fn read_cpu_governor(_cpu_id: usize) -> String {
    String::new()
}

/// Per-core temperatures: no OS-specific source on fallback targets; keep
/// `CpuInfo::temp_c` at `None` (the aggregate `SystemSnapshot::cpu_temp`
/// still flows from sysinfo `Components`).
pub fn read_core_temps(logical_cpus: usize) -> Vec<Option<f32>> {
    vec![None; logical_cpus]
}

pub fn read_mount_options() -> HashMap<String, String> {
    HashMap::new()
}

pub fn read_interface_ips() -> HashMap<String, Vec<String>> {
    HashMap::new()
}

pub fn read_batteries() -> Vec<BatteryInfo> {
    Vec::new()
}

pub fn read_thread_count(_pid: sysinfo::Pid) -> u64 {
    0
}

/// Directory Services users: not applicable on fallback targets; the kernel
/// merges nothing beyond `/etc/passwd`.
pub fn read_directory_users() -> Vec<(u32, String)> {
    Vec::new()
}

/// The numeric uid string sysinfo reports on unix (Windows resolves the
/// SID to its numeric RID in the windows module instead).
pub fn read_process_user_id(process: &sysinfo::Process) -> Option<String> {
    process.user_id().map(|uid| uid.to_string())
}

pub fn read_gpu_info_from_sysfs() -> Vec<GpuInfo> {
    Vec::new()
}

/// Package power (RAPL): no source on fallback targets; the provider keeps
/// `SystemInfo::package_power_w` at `None` (widgets hide the readout).
pub struct RaplPower;

impl Default for RaplPower {
    fn default() -> Self {
        Self
    }
}

impl RaplPower {
    pub fn new() -> Self {
        Self
    }

    pub fn sample(&mut self) -> Option<f64> {
        None
    }
}
