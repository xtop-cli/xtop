//! macOS battery probe from `/usr/bin/pmset -g batt`.
//!
//! `pmset` is present on every macOS install and returns instantly (it reads
//! the IOKit power source state). Running it per refresh keeps this probe
//! dependency-free; the parser below is pure and unit-tested.

use std::process::Command;

use xtop_plugin_api::model::BatteryInfo;

/// Battery state reported by `pmset`, mapped to the model contract used by
/// the battery widget (`"Charging"`/`"Discharging"` enable time readouts).
pub fn read_batteries() -> Vec<BatteryInfo> {
    let output = match Command::new("/usr/bin/pmset")
        .args(["-g", "batt"])
        .env("LC_ALL", "C")
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output);
    parse_pmset(&text)
}

/// Parse `pmset -g batt` output into batteries.
///
/// Line shape: ` -Name (id=1234)\tPERCENT%; state; [H:MM remaining] present: BOOL`
/// Percent and state fields always exist; the time field only appears while
/// charging/discharging. Batteries whose line says `present: false` (or
/// lacks the marker) are dropped.
fn parse_pmset(output: &str) -> Vec<BatteryInfo> {
    output.lines().filter_map(parse_battery_line).collect()
}

fn parse_battery_line(line: &str) -> Option<BatteryInfo> {
    if !line.contains("present: true") {
        return None;
    }
    // Name ends at the "(id=" group.
    let name = line
        .split("(id=")
        .next()?
        .trim()
        .trim_start_matches(['-', '*'])
        .to_string();
    if name.is_empty() {
        return None;
    }
    // Status segment: after the "(id=...)" group, before "present:".
    let tail = line.split("(id=").nth(1)?;
    let status = tail.split_once(')')?.1.split("present:").next()?.trim();

    let mut percent = None;
    let mut state = String::new();
    let mut remaining = None;
    for (i, part) in status.split(';').enumerate() {
        let part = part.trim();
        if i == 0 {
            percent = part.strip_suffix('%').and_then(|p| p.parse::<f32>().ok());
            continue;
        }
        if part.is_empty() {
            continue;
        }
        if let Some(rest) = part.strip_suffix(" remaining") {
            if let Some((h, m)) = rest.split_once(':') {
                if let (Ok(h), Ok(m)) = (h.parse::<u64>(), m.parse::<u64>()) {
                    remaining = Some(h * 60 + m);
                }
            }
            continue;
        }
        if state.is_empty() {
            state = part.to_string();
        }
    }

    let (model_state, time_to_full, time_to_empty) = match state.as_str() {
        "charging" => ("Charging", remaining, None),
        "discharging" => ("Discharging", None, remaining),
        "charged" => ("Full", None, None),
        "finishing charge" => ("Charging", remaining, None),
        "AC attached" => ("AC attached", None, None),
        "not charging" => ("Not charging", None, None),
        other => (other, None, None),
    };

    Some(BatteryInfo {
        name,
        percentage: percent.unwrap_or(0.0).clamp(0.0, 100.0),
        state: model_state.to_string(),
        time_to_full,
        time_to_empty,
        health: 0.0,
        cycle_count: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_discharging_battery_with_remaining_time() {
        let line =
            " -InternalBattery-0 (id=7077987)\t36%; discharging; 2:07 remaining present: true";
        let b = parse_battery_line(line).expect("battery parses");
        assert_eq!(b.name, "InternalBattery-0");
        assert_eq!(b.percentage, 36.0);
        assert_eq!(b.state, "Discharging");
        assert_eq!(b.time_to_full, None);
        assert_eq!(b.time_to_empty, Some(127));
    }

    #[test]
    fn parses_charging_battery_with_time_to_full() {
        let line = " -InternalBattery-0 (id=1)\t62%; charging; 0:43 remaining present: true";
        let b = parse_battery_line(line).expect("battery parses");
        assert_eq!(b.percentage, 62.0);
        assert_eq!(b.state, "Charging");
        assert_eq!(b.time_to_full, Some(43));
        assert_eq!(b.time_to_empty, None);
    }

    #[test]
    fn parses_full_battery_on_ac() {
        let line = " -InternalBattery-0 (id=1)\t100%; charged; 0:00 remaining present: true";
        let b = parse_battery_line(line).expect("battery parses");
        assert_eq!(b.percentage, 100.0);
        assert_eq!(b.state, "Full");
        assert_eq!(b.time_to_full, None);
        assert_eq!(b.time_to_empty, None);
    }

    #[test]
    fn parses_finishing_charge_and_ac_attached() {
        let line =
            " -InternalBattery-0 (id=1)\t99%; finishing charge; 0:00 remaining present: true";
        assert_eq!(parse_battery_line(line).unwrap().state, "Charging");
        let line = " -InternalBattery-0 (id=1)\t100%; AC attached; 0:00 remaining present: true";
        assert_eq!(parse_battery_line(line).unwrap().state, "AC attached");
    }

    #[test]
    fn drops_absent_batteries_and_malformed_lines() {
        let absent = " -InternalBattery-0 (id=1)\t--%; present: false";
        assert!(parse_battery_line(absent).is_none());
        assert!(parse_battery_line("completely malformed").is_none());
    }

    #[test]
    fn full_output_yields_all_present_batteries() {
        let text = "Now drawing from 'Battery Power'\n\
            -InternalBattery-0 (id=7077987)\t36%; discharging; 2:07 remaining present: true\n\
            -OtherBattery-1 (id=9)\t100%; charged; 0:00 remaining present: false\n";
        let bats = parse_pmset(text);
        assert_eq!(bats.len(), 1);
        assert_eq!(bats[0].name, "InternalBattery-0");
    }
}
