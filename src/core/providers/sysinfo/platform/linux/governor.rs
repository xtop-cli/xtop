//! Linux CPU governor probe from the `cpufreq` sysfs interface.

use std::fs;

/// CPU governor of a given core, empty when unavailable.
pub fn read_cpu_governor(cpu_id: usize) -> String {
    fs::read_to_string(format!(
        "/sys/devices/system/cpu/cpu{cpu_id}/cpufreq/scaling_governor"
    ))
    .map(|s| s.trim().to_string())
    .unwrap_or_default()
}
