//! Linux network interface address probe from `/proc/net`.

use std::collections::HashMap;
use std::fs;

/// Interface name to IP addresses.
pub fn read_interface_ips() -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    // IPv6 from /proc/net/if_inet6.
    if let Ok(content) = fs::read_to_string("/proc/net/if_inet6") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let addr_hex = parts[0];
                let iface = parts[4].to_string();
                if addr_hex.len() == 32 {
                    let ip: String = (0..8)
                        .map(|i| {
                            let start = i * 4;
                            let group = &addr_hex[start..start + 4];
                            let trimmed = group.trim_start_matches('0');
                            let val = u16::from_str_radix(
                                if trimmed.is_empty() { "0" } else { trimmed },
                                16,
                            )
                            .unwrap_or(0);
                            format!("{:x}", val)
                        })
                        .collect::<Vec<_>>()
                        .join(":");
                    map.entry(iface).or_default().push(ip);
                }
            }
        }
    }
    // IPv4 could be added from /proc/net/fib_trie if ever needed.
    map
}
