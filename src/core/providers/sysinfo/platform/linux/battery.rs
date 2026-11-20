//! Linux battery probes from `/sys/class/power_supply`.

use std::fs;
use std::path::Path;

use xtop_plugin_api::model::BatteryInfo;

/// Battery state from `/sys/class/power_supply`.
pub fn read_batteries() -> Vec<BatteryInfo> {
    let mut batteries = Vec::new();
    let power_supply = Path::new("/sys/class/power_supply");
    if !power_supply.exists() {
        return batteries;
    }
    if let Ok(entries) = fs::read_dir(power_supply) {
        for entry in entries.flatten() {
            let name = match entry.file_name().to_str() {
                Some(n) if n.starts_with("BAT") => n.to_string(),
                _ => continue,
            };
            let base = entry.path();
            let capacity = fs::read_to_string(base.join("capacity"))
                .ok()
                .and_then(|s| s.trim().parse::<f32>().ok())
                .unwrap_or(0.0);
            let state = fs::read_to_string(base.join("status"))
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            let charge_full = fs::read_to_string(base.join("charge_full"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            let charge_now = fs::read_to_string(base.join("charge_now"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            let cycles = fs::read_to_string(base.join("cycle_count"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
            let power_now = fs::read_to_string(base.join("power_now"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .unwrap_or(0);
            let charge_full_design = fs::read_to_string(base.join("charge_full_design"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(1);

            let health = if charge_full_design > 0 {
                (charge_full.unwrap_or(1) as f32 / charge_full_design as f32) * 100.0
            } else {
                100.0
            };

            // Time to full/empty estimated from the current power draw.
            let (time_to_full, time_to_empty) = if power_now != 0 && power_now.abs() > 0 {
                if state == "Charging" {
                    let remaining = charge_full
                        .unwrap_or(0)
                        .saturating_sub(charge_now.unwrap_or(0));
                    let secs = (remaining as f64 / power_now.abs() as f64 * 3600.0) as u64;
                    (Some(secs), None)
                } else if state == "Discharging" {
                    let secs =
                        (charge_now.unwrap_or(0) as f64 / power_now.abs() as f64 * 3600.0) as u64;
                    (None, Some(secs))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            batteries.push(BatteryInfo {
                name,
                percentage: capacity,
                state,
                time_to_full,
                time_to_empty,
                health,
                cycle_count: cycles,
            });
        }
    }
    batteries
}
