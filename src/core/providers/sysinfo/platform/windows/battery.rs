//! Windows battery probe via `GetSystemPowerStatus`.
//!
//! Windows reports one aggregate battery through this API (a single
//! `BatteryInfo` when present; desktops legitimately return an empty list).

use xtop_plugin_api::model::BatteryInfo;

use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

const ACLINE_ONLINE: u8 = 1;
const BATTERY_FLAG_CHARGING: u8 = 0x08;
const BATTERY_FLAG_NO_BATTERY: u8 = 0x80;
const BATTERY_UNKNOWN_PERCENT: u8 = 255;
const BATTERY_UNKNOWN_TIME: u32 = u32::MAX;

/// Battery state label and times from raw `SYSTEM_POWER_STATUS` fields.
///
/// Pure so the mapping is unit-tested with fixtures. `time_to_full` is
/// never reported by the API and `health`/`cycle_count` have no source, so
/// they stay default/`None` (widgets hide them, never fabricated).
fn map_battery_status(ac_line: u8, flag: u8, percent: u8, life_time: u32) -> Option<BatteryInfo> {
    if flag & BATTERY_FLAG_NO_BATTERY != 0 || percent == BATTERY_UNKNOWN_PERCENT {
        return None;
    }
    let state = if flag & BATTERY_FLAG_CHARGING != 0 {
        "Charging"
    } else if ac_line == ACLINE_ONLINE && percent == 100 {
        "Full"
    } else if ac_line == ACLINE_ONLINE {
        "Not charging"
    } else {
        "Discharging"
    };
    let time_to_empty = if state == "Discharging" && life_time != BATTERY_UNKNOWN_TIME {
        Some(life_time as u64)
    } else {
        None
    };
    Some(BatteryInfo {
        name: "Battery".to_string(),
        percentage: percent as f32,
        state: state.to_string(),
        time_to_full: None,
        time_to_empty,
        health: 0.0,
        cycle_count: None,
    })
}

/// Aggregate battery from `GetSystemPowerStatus`.
pub fn read_batteries() -> Vec<BatteryInfo> {
    let mut status = SYSTEM_POWER_STATUS {
        ACLineStatus: 0,
        BatteryFlag: 0,
        BatteryLifePercent: 0,
        SystemStatusFlag: 0,
        BatteryLifeTime: 0,
        BatteryFullLifeTime: 0,
    };
    let ok = unsafe { GetSystemPowerStatus(&mut status) };
    if ok == 0 {
        return Vec::new();
    }
    map_battery_status(
        status.ACLineStatus,
        status.BatteryFlag,
        status.BatteryLifePercent,
        status.BatteryLifeTime,
    )
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_maps_charging_ac_and_discharging_states() {
        let charging = map_battery_status(1, BATTERY_FLAG_CHARGING, 67, BATTERY_UNKNOWN_TIME)
            .expect("battery present");
        assert_eq!(charging.state, "Charging");
        assert_eq!(charging.percentage, 67.0);
        assert_eq!(charging.time_to_empty, None);
        assert_eq!(charging.time_to_full, None);

        let full = map_battery_status(1, 0, 100, BATTERY_UNKNOWN_TIME).expect("full");
        assert_eq!(full.state, "Full");

        let ac = map_battery_status(1, 0, 80, BATTERY_UNKNOWN_TIME).expect("ac-attached");
        assert_eq!(ac.state, "Not charging");

        let discharging = map_battery_status(0, 0, 45, 5400).expect("discharging with known time");
        assert_eq!(discharging.state, "Discharging");
        assert_eq!(discharging.time_to_empty, Some(5400));
        assert_eq!(discharging.health, 0.0, "no health source on Windows");
    }

    #[test]
    fn battery_absent_when_hardware_reports_none() {
        assert!(map_battery_status(255, 255, 255, BATTERY_UNKNOWN_TIME).is_none());
        assert!(
            map_battery_status(0, BATTERY_FLAG_NO_BATTERY, 255, BATTERY_UNKNOWN_TIME).is_none()
        );
    }
}
