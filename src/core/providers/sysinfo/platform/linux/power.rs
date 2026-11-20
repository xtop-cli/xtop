//! Linux package-power probe (UX9.1): Intel RAPL energy deltas.
//!
//! Linux exposes the CPU package energy through Intel RAPL as monotonically
//! increasing energy counters in microjoules. The probe reads the counters,
//! and the provider turns the delta between two reads at the refresh cadence
//! into an instantaneous package power in watts (`SystemInfo::package_power_w`).
//!
//! Sources read, in priority order:
//!
//! 1. `/sys/class/powercap/intel-rapl:<n>/energy_uj` for every RAPL domain
//!    whose `name` file reads `package-0` (one per CPU socket; readings are
//!    summed, so multi-socket machines report total package power). When no
//!    domain carries a `package-0` name, the lowest-index `intel-rapl:<n>`
//!    domain is used as the fallback.
//! 2. hwmon: the RAPL driver also registers a `powercap` hwmon device
//!    (`/sys/class/hwmon/hwmon*/name` == `powercap`) whose `energy*_input`
//!    files carry the same counters in microjoules; the first sensor only.
//!
//! Population rule (documented in the api data model): wattage is the energy
//! delta over the elapsed time since the previous sample
//! (`delta_uj / 1_000_000 / seconds`); counters wrap at
//! `max_energy_range_uj`, so deltas use wrapping subtraction. The first read
//! after startup establishes a baseline and yields `None` (no previous
//! counter yet). An unreadable source — no RAPL driver, a permission-denied
//! file (the `energy_uj` files are root-only on some distributions), or a
//! transient read failure — yields `None` and resets the baseline, so the
//! value is never fabricated and a counter that reappears re-baselines
//! cleanly. All other platforms stub the same API to `None`.

use std::fs;
use std::path::Path;
use std::time::Instant;

/// Stateful RAPL sampler: remembers the previous counter + timestamp and
/// turns each call into a watts reading. Kept in the provider so the probe
/// file stays stateless between refreshes (same pattern as the rate baselines
/// for disks/networks).
pub struct RaplPower {
    prev_energy_uj: Option<u64>,
    prev_at: Option<Instant>,
}

impl Default for RaplPower {
    fn default() -> Self {
        Self::new()
    }
}

impl RaplPower {
    pub fn new() -> Self {
        Self {
            prev_energy_uj: None,
            prev_at: None,
        }
    }

    /// Sample the instantaneous package power (watts) at the refresh
    /// cadence. `None` on the first call (baseline), when no readable RAPL
    /// source exists, or when the read fails — never fabricated.
    pub fn sample(&mut self) -> Option<f64> {
        let now = Instant::now();
        let Some(energy_uj) = read_energy_uj() else {
            // Unreadable source: drop the baseline so a later reappearing
            // counter starts a fresh delta instead of a bogus one.
            self.prev_energy_uj = None;
            self.prev_at = None;
            return None;
        };
        let watts = match (self.prev_energy_uj, self.prev_at) {
            (Some(prev), Some(at)) => {
                let elapsed = now.duration_since(at).as_secs_f64();
                if elapsed > 0.0 {
                    let delta = energy_uj.wrapping_sub(prev);
                    Some(energy_to_watts(delta, elapsed))
                } else {
                    None
                }
            }
            _ => None,
        };
        self.prev_energy_uj = Some(energy_uj);
        self.prev_at = Some(now);
        watts
    }
}

/// Pure delta math: `energy_delta_uj` over `elapsed_secs`, in watts.
fn energy_to_watts(energy_delta_uj: u64, elapsed_secs: f64) -> f64 {
    energy_delta_uj as f64 / 1_000_000.0 / elapsed_secs
}

/// Total package energy in microjoules across every readable RAPL source.
fn read_energy_uj() -> Option<u64> {
    // 1a. powercap package domains (preferred, one per socket).
    let packages = read_powercap_domains(Path::new("/sys/class/powercap"));
    if !packages.is_empty() {
        let total: u64 = packages.iter().filter_map(|p| read_energy_file(p)).sum();
        if total > 0 {
            return Some(total);
        }
    }
    // 1b. hwmon `powercap` device, first energy sensor.
    read_hwmon_energy()
}

/// Energy counter files of the RAPL domains whose `name` is `package-0`
/// (fallback: the lowest-index `intel-rapl:<n>` domain when no `package-0`
/// name exists). Sorted by directory index for determinism.
fn read_powercap_domains(root: &Path) -> Vec<std::path::PathBuf> {
    let mut named = Vec::new();
    let mut fallback: Option<std::path::PathBuf> = None;
    let Ok(entries) = fs::read_dir(root) else {
        return named;
    };
    let mut dirs: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .is_some_and(|f| f.starts_with("intel-rapl:"))
        })
        .collect();
    dirs.sort();
    for dir in dirs {
        let name = fs::read_to_string(dir.join("name")).unwrap_or_default();
        if name.trim() == "package-0" {
            named.push(dir.clone());
        }
        if fallback.is_none() && dir.join("energy_uj").is_file() {
            fallback = Some(dir);
        }
    }
    if !named.is_empty() {
        named
    } else {
        fallback.into_iter().collect()
    }
}

fn read_energy_file(dir: &Path) -> Option<u64> {
    fs::read_to_string(dir.join("energy_uj"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
}

/// hwmon fallback: first `energy*_input` of a `powercap` hwmon device.
fn read_hwmon_energy() -> Option<u64> {
    let hwmon = Path::new("/sys/class/hwmon");
    let mut devices: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(hwmon) {
        devices = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                fs::read_to_string(p.join("name"))
                    .map(|n| n.trim() == "powercap")
                    .unwrap_or(false)
            })
            .collect();
    }
    devices.sort();
    for dir in devices {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut sensors: Vec<std::path::PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|f| f.to_str())
                    .is_some_and(|f| f.starts_with("energy") && f.ends_with("_input"))
            })
            .collect();
        sensors.sort();
        for sensor in sensors {
            if let Ok(v) = fs::read_to_string(&sensor) {
                if let Ok(v) = v.trim().parse::<u64>() {
                    if v > 0 {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_math_converts_microjoules_to_watts() {
        // 60 J over 1 s == 60 W; 60 J over 2 s == 30 W.
        assert_eq!(energy_to_watts(60_000_000, 1.0), 60.0);
        assert_eq!(energy_to_watts(60_000_000, 2.0), 30.0);
        // 1 J over 250 ms == 4 W.
        assert_eq!(energy_to_watts(1_000_000, 0.25), 4.0);
        // Wrap-around delta (counter reset at max_energy_range_uj) still
        // yields a sane small reading instead of a huge one.
        assert_eq!(energy_to_watts(1_000, 1.0), 0.001);
    }

    #[test]
    fn powercap_package_domain_selection_skips_non_package_domains() {
        // The fixture mirrors intel-rapl:0 (package-0) with core/uncore
        // subdomains: only the package domain is selectable, and a missing
        // package-0 falls back to the lowest-index domain.
        let tmp = std::env::temp_dir().join("xtop-rapl-test");
        let _ = fs::remove_dir_all(&tmp);
        let root = tmp.join("powercap");
        for (sub, name) in [
            ("intel-rapl:0", "package-0"),
            ("intel-rapl:0:0", "core"),
            ("intel-rapl:0:1", "uncore"),
            ("intel-rapl:1", "package-0"),
        ] {
            let dir = root.join(sub);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("name"), name).unwrap();
        }
        let dirs = read_powercap_domains(&root);
        let names: Vec<String> = dirs
            .iter()
            .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, ["intel-rapl:0", "intel-rapl:1"]);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn powercap_falls_back_to_lowest_index_without_package_name() {
        let tmp = std::env::temp_dir().join("xtop-rapl-test-fallback");
        let _ = fs::remove_dir_all(&tmp);
        let root = tmp.join("powercap");
        for (sub, name) in [("intel-rapl:1", "dram"), ("intel-rapl:0", "dram")] {
            let dir = root.join(sub);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("name"), name).unwrap();
            // The fallback rule requires a readable energy counter file.
            fs::write(dir.join("energy_uj"), "0").unwrap();
        }
        let dirs = read_powercap_domains(&root);
        let names: Vec<String> = dirs
            .iter()
            .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, ["intel-rapl:0"]);
        let _ = fs::remove_dir_all(&tmp);
    }
}
