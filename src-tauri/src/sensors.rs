use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct SensorSnapshot {
    pub battery_percent: Option<u8>,
    pub battery_charging: Option<bool>,
    pub temperatures: Vec<TemperatureReading>,
}

#[derive(Serialize, Clone)]
pub struct TemperatureReading {
    pub label: String,
    pub celsius: f32,
}

pub fn parse_capacity(content: &str) -> Option<u8> {
    content.trim().parse::<u8>().ok()
}

fn read_battery() -> (Option<u8>, Option<bool>) {
    let base = Path::new("/sys/class/power_supply/BAT0");
    let capacity = fs::read_to_string(base.join("capacity"))
        .ok()
        .and_then(|s| parse_capacity(&s));
    let status = fs::read_to_string(base.join("status"))
        .ok()
        .map(|s| s.trim().eq_ignore_ascii_case("charging"));
    (capacity, status)
}

fn read_temperatures() -> Vec<TemperatureReading> {
    use sysinfo::Components;
    let components = Components::new_with_refreshed_list();
    components
        .iter()
        .filter_map(|c| {
            let celsius = c.temperature();
            if celsius.is_nan() {
                None
            } else {
                Some(TemperatureReading {
                    label: c.label().to_string(),
                    celsius,
                })
            }
        })
        .collect()
}

#[tauri::command]
pub fn get_sensor_snapshot() -> SensorSnapshot {
    let (battery_percent, battery_charging) = read_battery();
    SensorSnapshot {
        battery_percent,
        battery_charging,
        temperatures: read_temperatures(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_battery_capacity_from_sysfs_content() {
        assert_eq!(parse_capacity("87\n"), Some(87));
        assert_eq!(parse_capacity(""), None);
        assert_eq!(parse_capacity("not-a-number"), None);
    }
}
