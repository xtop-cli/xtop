//! macOS mount option probe.
//!
//! macOS has no `/proc/self/mountinfo`. `/sbin/mount` (no args) prints the
//! live mount table as parsed by the kernel, one line per mount:
//!
//! `/dev/disk3s1 on / (apfs, local, read-only, journaled)`
//!
//! The parser is pure and unit-tested; only the command invocation is
//! platform-specific.

use std::collections::HashMap;
use std::process::Command;

/// Mount option string per mount point.
pub fn read_mount_options() -> HashMap<String, String> {
    let output = match Command::new("/sbin/mount").env("LC_ALL", "C").output() {
        Ok(out) if out.status.success() => out.stdout,
        _ => return HashMap::new(),
    };
    let text = String::from_utf8_lossy(&output);
    parse_mount_output(&text)
}

/// Parse `mount(8)` output lines of the form
/// `device on mount_point (option, list)`, skipping anything else.
fn parse_mount_output(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for raw in output.lines() {
        let line = raw.trim();
        let Some(rest) = line.find(" on ") else {
            continue;
        };
        let after = &line[rest + 4..];
        let Some(open) = after.find(" (") else {
            continue;
        };
        let mount_point = after[..open].trim().to_string();
        if mount_point.is_empty() {
            continue;
        }
        let Some(close) = after[open + 2..].find(')') else {
            continue;
        };
        let options = after[open + 2..open + 2 + close].trim().to_string();
        map.insert(mount_point, options);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apfs_and_devfs_mounts() {
        let text = "/dev/disk5s2s1 on / (apfs, sealed, local, read-only, journaled)\n\
            devfs on /dev (devfs, local, nobrowse)\n\
            /dev/disk5s6 on /System/Volumes/VM (apfs, local, noexec, journaled, noatime, nobrowse)\n";
        let map = parse_mount_output(text);
        assert_eq!(
            map.get("/").map(String::as_str),
            Some("apfs, sealed, local, read-only, journaled")
        );
        assert_eq!(
            map.get("/dev").map(String::as_str),
            Some("devfs, local, nobrowse")
        );
        assert_eq!(
            map.get("/System/Volumes/VM").map(String::as_str),
            Some("apfs, local, noexec, journaled, noatime, nobrowse")
        );
    }

    #[test]
    fn skips_non_mount_lines() {
        let map = parse_mount_output("auto_home on /System/Volumes/Data/home (autofs, automounted, nobrowse)\nrandom noise\n");
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("/System/Volumes/Data/home"));
    }
}
