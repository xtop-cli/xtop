//! macOS probes.
//!
//! Submodules map one-to-one to the probes of the linux backend:
//!
//! - [`battery`]: power state via `/usr/bin/pmset -g batt`.
//! - [`interfaces`]: per-interface IP addresses via `getifaddrs`.
//! - [`mounts`]: mount option strings via `/sbin/mount`.
//! - [`threads`]: per-process thread counts via `proc_pidinfo`.
//! - [`users`]: Directory Services login names via `/usr/bin/dscl`.
//!
//! Probes that have no meaningful macOS source stay empty per the shared
//! contract ("never fabricate a reading"):
//!
//! - Per-core temperatures: macOS has no public per-logical-core sensor
//!   keys (Apple Silicon reports P/E cluster and proximity values only), so
//!   `CpuInfo::temp_c` stays `None`. The aggregate
//!   `SystemSnapshot::cpu_temp` still flows from the sysinfo crate (AppleSMC
//!   on Intel, HID temperature events on Apple Silicon).
//! - Package power: `package_power_w` stays `None` (no RAPL; SMC power keys
//!   are Intel-only and model-specific).
//! - GPUs: Apple exposes no public per-GPU utilization/temperature API;
//!   the shared `nvidia-smi` probe still covers NVIDIA hardware. The GPU
//!   widget renders an honest empty state otherwise.
//! - CPU governor: macOS has no cpufreq equivalent; the value stays empty.

pub mod battery;
pub mod interfaces;
pub mod mounts;
pub mod threads;
pub mod users;

use xtop_plugin_api::model::GpuInfo;

pub use battery::read_batteries;
pub use interfaces::read_interface_ips;
pub use mounts::read_mount_options;
pub use threads::read_thread_count;
pub use users::read_directory_users;

/// CPU governor: not applicable on macOS (no cpufreq interface).
pub fn read_cpu_governor(_cpu_id: usize) -> String {
    String::new()
}

/// Per-core temperatures: see the module docs for why this stays `None`.
pub fn read_core_temps(logical_cpus: usize) -> Vec<Option<f32>> {
    vec![None; logical_cpus]
}

/// GPU info: see the module docs for why this stays empty.
pub fn read_gpu_info_from_sysfs() -> Vec<GpuInfo> {
    Vec::new()
}

/// The numeric uid string sysinfo reports on unix (Windows resolves the
/// SID to its numeric RID in the windows module instead).
pub fn read_process_user_id(process: &sysinfo::Process) -> Option<String> {
    process.user_id().map(|uid| uid.to_string())
}

/// Package power: see the module docs for why this stays `None`.
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

#[cfg(test)]
mod live_tests {
    //! Live smoke tests against the current macOS host. These exercise the
    //! FFI paths (getifaddrs, proc_pidinfo, subprocess probes) that the pure
    //! unit tests cannot reach. Compiled on macOS only.

    use super::*;

    #[test]
    fn live_mount_options_include_root() {
        assert!(read_mount_options().contains_key("/"));
    }

    #[test]
    fn live_thread_count_of_self_is_positive() {
        let pid = sysinfo::get_current_pid().expect("current pid");
        assert!(read_thread_count(pid) > 0);
    }

    #[test]
    fn live_interfaces_are_enumerated() {
        let ips = read_interface_ips();
        assert!(!ips.is_empty(), "loopback alone must appear");
        assert!(
            ips.values().flatten().any(|ip| ip.contains(':')),
            "IPv6 loopback (::1) expected"
        );
    }

    #[test]
    fn live_directory_users_are_resolved() {
        assert!(!read_directory_users().is_empty());
    }

    #[test]
    fn live_battery_probe_does_not_crash() {
        // Desktops may legitimately return an empty list.
        let _ = read_batteries();
    }
}
