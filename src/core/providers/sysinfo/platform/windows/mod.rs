//! Windows probes.
//!
//! Submodules map one-to-one to the probes of the linux backend:
//!
//! - [`battery`]: aggregate battery status via `GetSystemPowerStatus`.
//! - [`interfaces`]: per-interface IP addresses via `GetAdaptersAddresses`
//!   (keyed by the adapter friendly name, the name sysinfo uses for its
//!   network list on Windows).
//! - [`mounts`]: per-volume options from `GetLogicalDrives` +
//!   `GetVolumeInformationW`.
//! - [`threads`]: per-process thread counts via toolhelp snapshots.
//! - [`users`]: local accounts via `Get-LocalUser` (startup-only) and the
//!   SID → numeric RID formatting for process user ids.
//!
//! Probes that have no meaningful Windows source stay empty per the shared
//! contract ("never fabricate a reading"):
//!
//! - Per-core temperatures: Windows exposes thermal zones (WMI
//!   `MSAcpi_ThermalZoneTemperature`), never per-logical-core sensors, so
//!   `CpuInfo::temp_c` stays `None`. The aggregate
//!   `SystemSnapshot::cpu_temp` still flows from sysinfo `Components` (the
//!   same WMI class) when the host exposes a zone.
//! - CPU governor: no cpufreq equivalent on Windows; the value stays empty.
//! - GPUs: no public AMD/Intel utilization API (same as macOS); the shared
//!   `nvidia-smi` probe still covers NVIDIA hardware. The GPU widget
//!   renders an honest empty state otherwise.
//! - Package power: `package_power_w` stays `None` (no RAPL; the
//!   `Win32_PowerMeter` WMI class is not universally readable).

mod battery;
mod interfaces;
mod mounts;
mod threads;
mod users;

use xtop_plugin_api::model::GpuInfo;

pub use battery::read_batteries;
pub use interfaces::read_interface_ips;
pub use mounts::read_mount_options;
pub use threads::read_thread_count;
pub use users::{read_directory_users, read_process_user_id};

/// CPU governor: not applicable on Windows (power schemes are system-wide
/// and nothing renders the value).
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

/// UTF-16 with a trailing NUL for Win32 wide-string parameters.
pub(crate) fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Read a NUL-terminated wide string from a raw pointer.
pub(crate) unsafe fn wide_str(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_strings_are_nul_terminated() {
        let encoded = wide("C:\\");
        assert_eq!(encoded, vec![b'C' as u16, b':' as u16, b'\\' as u16, 0]);
        unsafe {
            assert_eq!(wide_str(encoded.as_ptr()), "C:\\");
            assert!(wide_str(std::ptr::null()).is_empty());
        }
    }

    mod live_tests {
        //! Live smoke tests against the current Windows host, mirroring the
        //! macOS `live_tests` module: they exercise the FFI paths that pure
        //! unit tests cannot reach. Compiled on Windows only.

        use super::*;

        #[test]
        fn live_mount_options_include_current_drive() {
            use std::path::Component;
            let cwd = std::env::current_dir().expect("cwd");
            let root = cwd
                .components()
                .find_map(|part| match part {
                    Component::Prefix(prefix) => {
                        Some(format!("{}\\", prefix.as_os_str().to_string_lossy()))
                    }
                    _ => None,
                })
                .expect("windows path has a drive prefix");
            assert!(
                read_mount_options().contains_key(&root),
                "current drive {root} expected in {:#?}",
                read_mount_options()
            );
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

        #[test]
        fn live_current_process_uid_is_numeric() {
            let me = sysinfo::get_current_pid().expect("current pid");
            let all = sysinfo::System::new_all();
            let process = all.process(me).expect("self process");
            let uid = read_process_user_id(process).expect("current user id");
            assert!(
                uid.parse::<u32>().is_ok() && !uid.contains('-'),
                "user id must be the numeric RID, got {uid}"
            );
        }
    }
}
