//! Linux per-core temperature probe (UX8.3).
//!
//! Linux exposes per-core temperatures through the `coretemp` hwmon driver:
//! `/sys/class/hwmon/hwmon*/name` reads `coretemp`, each `temp*_label` is
//! `Core N` and the matching `temp*_input` carries the temperature in
//! millidegrees Celsius. The sensor index `N` numbers the physical cores of
//! the package; every logical CPU maps to its physical core through
//! `/sys/devices/system/cpu/cpuN/topology/core_id`.
//!
//! Population rule (documented in the api data model): `CpuInfo::temp_c` is
//! filled per logical core only when the mapping is unambiguous — exactly
//! one `coretemp` device (single package), a dense sensor set `Core 0..P-1`,
//! and logical `core_id`s dense over `0..P-1` (SMT siblings share their
//! physical core's temperature). Any other machine (no coretemp driver,
//! multiple packages, unreadable topology) yields `None` per logical core;
//! the aggregate component temperature keeps flowing through
//! `SystemSnapshot::cpu_temp` (sysinfo `Components`) for the header readout.

use std::fs;
use std::path::Path;

/// Read the temperature of the physical core backing each logical CPU.
///
/// The returned vector is indexed like the provider's `cpus()` list: entry
/// `i` is the °C reading for logical CPU `i`, or `None` when the platform
/// does not expose an unambiguous per-core mapping.
pub fn read_core_temps(logical_cpus: usize) -> Vec<Option<f32>> {
    let mut sensors = Vec::new();
    let mut devices = 0usize;

    let hwmon = Path::new("/sys/class/hwmon");
    if let Ok(entries) = fs::read_dir(hwmon) {
        for entry in entries.flatten() {
            let dir = entry.path();
            let name = fs::read_to_string(dir.join("name")).unwrap_or_default();
            if name.trim() != "coretemp" {
                continue;
            }
            devices += 1;
            collect_core_sensors(&dir, &mut sensors);
        }
    }

    // Only a single-package coretemp set can be mapped to logical cores.
    if devices != 1 || sensors.is_empty() {
        return vec![None; logical_cpus];
    }

    let mut core_ids = Vec::with_capacity(logical_cpus);
    for cpu in 0..logical_cpus {
        let id = fs::read_to_string(format!("/sys/devices/system/cpu/cpu{cpu}/topology/core_id"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        core_ids.push(id);
    }

    map_core_temps(&core_ids, &sensors)
}

/// Parse `temp*_label` = `Core N` + `temp*_input` (millidegrees) pairs.
fn collect_core_sensors(dir: &Path, out: &mut Vec<(u32, f32)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut labels: Vec<(u32, String)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file) = path.file_name().and_then(|f| f.to_str()).map(String::from) else {
            continue;
        };
        if !file.starts_with("temp") || !file.ends_with("_label") {
            continue;
        }
        let index = &file[4..file.len() - "_label".len()];
        if !index.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let label = fs::read_to_string(&path).unwrap_or_default();
        let core = label
            .trim()
            .strip_prefix("Core ")
            .and_then(|n| n.parse::<u32>().ok());
        if let Some(core) = core {
            labels.push((core, file));
        }
    }
    for (core, label_file) in labels {
        let input_file = format!("temp{}_input", &label_file[4..label_file.len() - 6]);
        if let Ok(milli) = fs::read_to_string(dir.join(&input_file)) {
            if let Ok(milli) = milli.trim().parse::<f32>() {
                out.push((core, milli / 1000.0));
            }
        }
    }
}

/// Pure mapping: `logical_core_ids[i]` is the `core_id` of logical CPU `i`
/// (`None` when unreadable); `sensors` are `(core index, °C)` pairs. Returns
/// `Some(°C)` per logical CPU only when the mapping is dense and exact.
fn map_core_temps(logical_core_ids: &[Option<u32>], sensors: &[(u32, f32)]) -> Vec<Option<f32>> {
    let mut sorted = sensors.to_vec();
    sorted.sort_by_key(|(core, _)| *core);
    let dense_sensors = (0..sorted.len()).all(|k| sorted[k].0 == k as u32);
    if !dense_sensors || sorted.is_empty() {
        return vec![None; logical_core_ids.len()];
    }

    let mut ids: Vec<u32> = logical_core_ids.iter().flatten().copied().collect();
    ids.sort_unstable();
    ids.dedup();
    let dense_ids = ids.len() == sorted.len() && (0..ids.len()).all(|k| ids[k] == k as u32);
    if !dense_ids {
        return vec![None; logical_core_ids.len()];
    }

    logical_core_ids
        .iter()
        .map(|id| id.and_then(|core| sorted.get(core as usize).map(|(_, t)| *t)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(v: &[Option<u32>]) -> Vec<Option<u32>> {
        v.to_vec()
    }

    #[test]
    fn dense_mapping_attributes_sibling_cores() {
        // 4 logical CPUs, SMT pairs 0-1 and 2-3 on physical cores 0 and 1.
        let ids = id(&[Some(0), Some(0), Some(1), Some(1)]);
        let sensors = vec![(0, 55.0), (1, 61.0)];
        let temps = map_core_temps(&ids, &sensors);
        assert_eq!(temps, vec![Some(55.0), Some(55.0), Some(61.0), Some(61.0)]);
    }

    #[test]
    fn sparse_sensor_set_yields_none() {
        let ids = id(&[Some(0), Some(0)]);
        // Missing "Core 1" sensor: cannot map core 1.
        let sensors = vec![(0, 55.0), (2, 70.0)];
        assert!(map_core_temps(&ids, &sensors).iter().all(Option::is_none));
    }

    #[test]
    fn unreadable_core_id_yields_none_for_that_cpu() {
        let ids = id(&[Some(0), None, Some(1)]);
        let sensors = vec![(0, 55.0), (1, 61.0)];
        let temps = map_core_temps(&ids, &sensors);
        assert_eq!(temps[0], Some(55.0));
        assert_eq!(temps[1], None);
        assert_eq!(temps[2], Some(61.0));
    }

    #[test]
    fn duplicate_or_out_of_range_sensors_yield_none() {
        let ids = id(&[Some(0), Some(1)]);
        let sensors = vec![(1, 61.0), (1, 62.0)];
        assert!(map_core_temps(&ids, &sensors).iter().all(Option::is_none));
        let ids = id(&[Some(0), Some(1)]);
        let sensors = vec![(0, 55.0)];
        assert_eq!(map_core_temps(&ids, &sensors).len(), 2);
    }
}
