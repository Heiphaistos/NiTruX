use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Scans `base` for the first `BAT*`-prefixed subdirectory, sorted
/// lexicographically (so BAT0 is preferred over BAT1 when both exist).
/// Returns None if no battery directory is present at all (desktops,
/// or laptops where the kernel doesn't expose one this way).
pub fn find_battery_dir_in(base: &Path) -> Option<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(base)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("BAT"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    entries.into_iter().next()
}

fn find_battery_dir() -> Option<PathBuf> {
    find_battery_dir_in(Path::new("/sys/class/power_supply"))
}

fn read_battery() -> (Option<u8>, Option<bool>) {
    let Some(base) = find_battery_dir() else {
        return (None, None);
    };
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

    #[test]
    fn finds_first_available_battery_dir_preferring_lower_numbers() {
        // find_battery_dir scans /sys/class/power_supply for entries starting
        // with "BAT", returning the lexicographically-first match (BAT0 before
        // BAT1) when multiple exist. This test uses a fake base directory
        // populated with fixture subdirectories rather than touching the real
        // /sys, so it works identically on any machine including this dev host.
        let base = std::env::temp_dir().join(format!("nitrux-batt-test-{}", std::process::id()));
        std::fs::create_dir_all(base.join("BAT1")).unwrap();
        std::fs::create_dir_all(base.join("BAT0")).unwrap();
        std::fs::create_dir_all(base.join("AC")).unwrap();

        let found = find_battery_dir_in(&base);
        std::fs::remove_dir_all(&base).ok();

        assert_eq!(found, Some(base.join("BAT0")));
    }

    #[test]
    fn returns_none_when_no_battery_present() {
        let base = std::env::temp_dir().join(format!("nitrux-batt-empty-{}", std::process::id()));
        std::fs::create_dir_all(base.join("AC")).unwrap();

        let found = find_battery_dir_in(&base);
        std::fs::remove_dir_all(&base).ok();

        assert_eq!(found, None);
    }
}
