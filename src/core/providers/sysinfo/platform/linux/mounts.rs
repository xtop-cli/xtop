//! Linux mount option probe from `/proc/self/mountinfo`.

use std::collections::HashMap;
use std::fs;

/// Mount option string per mount point.
///
/// Format per line: `id parent_id maj:min root mount_point options ...`
pub fn read_mount_options() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string("/proc/self/mountinfo") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount_point = parts[4].to_string();
                let opts = parts[5].to_string();
                map.insert(mount_point, opts);
            }
        }
    }
    map
}
