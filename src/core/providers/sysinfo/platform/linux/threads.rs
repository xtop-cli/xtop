//! Linux process thread count probe from `/proc/<pid>/status`.

use std::fs;

/// Thread count of a process.
pub fn read_thread_count(pid: sysinfo::Pid) -> u64 {
    let path = format!("/proc/{pid}/status");
    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("Threads:\t") {
                return rest.trim().parse::<u64>().unwrap_or(0);
            }
        }
    }
    0
}
