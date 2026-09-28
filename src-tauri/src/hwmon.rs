//! Every hardware sensor the kernel exposes, read straight from
//! `/sys/class/hwmon`: temperatures, fan and pump speeds, PWM duty,
//! voltages, currents, power and energy. No external tool and no root, so
//! it works on any distribution, any init system and any C library, and it
//! covers what `lm-sensors` would show plus chips it does not know by name
//! (NVMe drives, AMD/Intel GPUs, DDR5 DIMMs, USB liquid coolers, Corsair
//! PSUs...). Chips are classified by driver name so the UI can say "Carte
//! graphique" or "Watercooling" instead of "amdgpu" or "nzxt-kraken3".

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SensorReading {
    /// "temperature", "fan", "pwm", "voltage", "current", "power",
    /// "energy" or "humidity".
    pub kind: String,
    pub label: String,
    pub value: f64,
    /// "°C", "RPM", "%", "V", "A", "W", "J", "%HR".
    pub unit: String,
    pub max: Option<f64>,
    pub critical: Option<f64>,
    pub alarm: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SensorChip {
    /// Kernel driver name (`coretemp`, `amdgpu`, `nvme`...).
    pub chip: String,
    /// "cpu", "gpu", "storage", "memory", "motherboard", "cooling", "psu",
    /// "battery", "network" or "other".
    pub category: String,
    /// Human-readable device: drive model, GPU name, PCI address...
    pub device: String,
    pub readings: Vec<SensorReading>,
}

/// Maps a hwmon driver name to the component it measures.
pub fn classify_chip(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    let starts = |prefixes: &[&str]| prefixes.iter().any(|p| n.starts_with(p));
    if starts(&["coretemp", "k10temp", "k8temp", "zenpower", "fam15h_power", "cpu_thermal", "x86_pkg_temp", "via_cputemp", "cpu-thermal", "soc_thermal"]) {
        "cpu"
    } else if starts(&["amdgpu", "radeon", "nouveau", "i915", "xe", "nvidia", "gpu_thermal", "panfrost", "gpu-thermal"]) {
        "gpu"
    } else if starts(&["nvme", "drivetemp", "sata", "hdd"]) {
        "storage"
    } else if starts(&["spd5118", "jc42", "ee1004", "dimm"]) {
        "memory"
    } else if starts(&["kraken", "nzxt", "corsaircpro", "corsair-cpro", "corsair_cpro", "aquacomputer", "d5next", "quadro", "octo", "highflow", "farbwerk", "asus_rog_ryujin", "ryujin", "leakshield", "powerjoy", "corsairpsu_ignored"]) {
        "cooling"
    } else if starts(&["corsairpsu", "corsair-psu", "corsair_psu", "psu"]) {
        "psu"
    } else if starts(&["bat", "battery", "acpi_battery", "ac", "ucsi", "sbs"]) {
        "battery"
    } else if starts(&["iwlwifi", "mt76", "ath", "r8169", "igc", "ixgbe", "mlx", "bnxt", "phy"]) {
        "network"
    } else if starts(&["nct", "it8", "it87", "f71", "f75", "w83", "asus", "dell", "thinkpad", "hp", "acpitz", "gigabyte", "sch56", "lm", "adt", "pch_", "wmi", "applesmc", "lenovo", "msi", "ideapad", "surface", "cros_ec", "smsc", "fintek", "atk"]) {
        "motherboard"
    } else {
        "other"
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn read_number(path: &Path) -> Option<f64> {
    read_trimmed(path)?.parse::<f64>().ok()
}

/// (kind, unit, divisor from the sysfs integer to the display unit)
fn channel_kind(prefix: &str) -> Option<(&'static str, &'static str, f64)> {
    Some(match prefix {
        "temp" => ("temperature", "°C", 1000.0),
        "fan" => ("fan", "RPM", 1.0),
        "in" => ("voltage", "V", 1000.0),
        "curr" => ("current", "A", 1000.0),
        "power" => ("power", "W", 1_000_000.0),
        "energy" => ("energy", "J", 1_000_000.0),
        "humidity" => ("humidity", "%HR", 1000.0),
        _ => return None,
    })
}

/// Splits `temp3_input` into ("temp", 3, "input"); `pwm2` into ("pwm", 2, "").
pub fn split_attribute(file: &str) -> Option<(String, u32, String)> {
    let (head, attr) = file.split_once('_').unwrap_or((file, ""));
    let digits_at = head.find(|c: char| c.is_ascii_digit())?;
    let (prefix, index) = head.split_at(digits_at);
    Some((prefix.to_string(), index.parse().ok()?, attr.to_string()))
}

/// Reads one hwmon directory (`/sys/class/hwmon/hwmonN`, or any directory
/// with the same layout -- tests use a temporary one).
pub fn read_chip_dir(dir: &Path) -> Option<SensorChip> {
    let chip = read_trimmed(&dir.join("name")).unwrap_or_else(|| "inconnu".to_string());
    let mut channels: Vec<(String, u32)> = Vec::new();
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        if let Some((prefix, index, attr)) = split_attribute(&file) {
            let is_value = (attr == "input" || (prefix == "power" && attr == "average")) || (prefix == "pwm" && attr.is_empty());
            if is_value && !channels.contains(&(prefix.clone(), index)) {
                channels.push((prefix, index));
            }
        }
    }
    channels.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut readings = Vec::new();
    for (prefix, index) in channels {
        let base = format!("{prefix}{index}");
        if prefix == "pwm" {
            if let Some(raw) = read_number(&dir.join(&base)) {
                readings.push(SensorReading {
                    kind: "pwm".into(),
                    label: format!("PWM {index}"),
                    value: (raw / 255.0 * 100.0).round(),
                    unit: "%".into(),
                    max: None,
                    critical: None,
                    alarm: false,
                });
            }
            continue;
        }
        let Some((kind, unit, div)) = channel_kind(&prefix) else { continue };
        let value = read_number(&dir.join(format!("{base}_input")))
            .or_else(|| read_number(&dir.join(format!("{base}_average"))));
        let Some(raw) = value else { continue };
        // A disconnected fan header reads 0 RPM and a missing probe often
        // reads -128 °C or a huge value: neither is a measurement.
        if kind == "temperature" && !(-60_000.0..=250_000.0).contains(&raw) {
            continue;
        }
        let label = read_trimmed(&dir.join(format!("{base}_label"))).unwrap_or_else(|| default_label(kind, index));
        let scaled = |name: &str| read_number(&dir.join(format!("{base}_{name}"))).map(|v| v / div).filter(|v| *v > 0.0);
        let alarm = read_number(&dir.join(format!("{base}_alarm"))).is_some_and(|a| a > 0.0)
            || read_number(&dir.join(format!("{base}_crit_alarm"))).is_some_and(|a| a > 0.0);
        readings.push(SensorReading {
            kind: kind.into(),
            label,
            value: (raw / div * 100.0).round() / 100.0,
            unit: unit.into(),
            max: scaled("max"),
            critical: scaled("crit"),
            alarm,
        });
    }
    if readings.is_empty() {
        return None;
    }
    Some(SensorChip { category: classify_chip(&chip).to_string(), device: describe_device(dir, &chip), chip, readings })
}

fn default_label(kind: &str, index: u32) -> String {
    match kind {
        "temperature" => format!("Température {index}"),
        "fan" => format!("Ventilateur {index}"),
        "voltage" => format!("Tension {index}"),
        "current" => format!("Courant {index}"),
        "power" => format!("Puissance {index}"),
        "energy" => format!("Énergie {index}"),
        _ => format!("Capteur {index}"),
    }
}

/// Names the physical device behind a chip: the drive model for NVMe and
/// SATA (`drivetemp`) sensors, the PCI address for GPUs, the USB product
/// for liquid coolers, the power-supply name for batteries.
fn describe_device(dir: &Path, chip: &str) -> String {
    let device = dir.join("device");
    for candidate in ["model", "product", "name"] {
        if let Some(v) = read_trimmed(&device.join(candidate)) {
            if v != chip {
                return v;
            }
        }
    }
    // USB devices keep `product` one level up from the HID interface.
    if let Some(v) = read_trimmed(&device.join("../product")) {
        return v;
    }
    fs::canonicalize(&device)
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
        .unwrap_or_default()
}

/// Fallback for systems without hwmon temperature drivers (many ARM boards,
/// some VMs): the generic thermal zones.
fn read_thermal_zones(base: &Path) -> Vec<SensorChip> {
    let Ok(entries) = fs::read_dir(base) else { return Vec::new() };
    let mut zones: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.join("temp").exists()).collect();
    zones.sort();
    zones
        .into_iter()
        .filter_map(|zone| {
            let kind = read_trimmed(&zone.join("type")).unwrap_or_else(|| "thermal".into());
            let raw = read_number(&zone.join("temp"))?;
            Some(SensorChip {
                category: classify_chip(&kind).to_string(),
                device: zone.file_name()?.to_string_lossy().into_owned(),
                readings: vec![SensorReading {
                    kind: "temperature".into(),
                    label: kind.clone(),
                    value: (raw / 10.0).round() / 100.0,
                    unit: "°C".into(),
                    max: None,
                    critical: None,
                    alarm: false,
                }],
                chip: kind,
            })
        })
        .collect()
}

pub fn read_all_sensors_in(hwmon_base: &Path, thermal_base: &Path) -> Vec<SensorChip> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(hwmon_base).map(|e| e.flatten().map(|e| e.path()).collect()).unwrap_or_default();
    // hwmon10 after hwmon9, not after hwmon1.
    dirs.sort_by_key(|p| {
        p.file_name().and_then(|f| f.to_str()).and_then(|f| f.trim_start_matches("hwmon").parse::<u32>().ok()).unwrap_or(u32::MAX)
    });
    let mut chips: Vec<SensorChip> = dirs.iter().filter_map(|d| read_chip_dir(d)).collect();
    if !chips.iter().any(|c| c.readings.iter().any(|r| r.kind == "temperature")) {
        chips.extend(read_thermal_zones(thermal_base));
    }
    let order = ["cpu", "gpu", "storage", "memory", "motherboard", "cooling", "psu", "battery", "network", "other"];
    chips.sort_by_key(|c| order.iter().position(|o| *o == c.category).unwrap_or(order.len()));
    chips
}

#[tauri::command]
pub fn get_all_sensors() -> Vec<SensorChip> {
    read_all_sensors_in(Path::new("/sys/class/hwmon"), Path::new("/sys/class/thermal"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_chip(root: &Path, dir: &str, files: &[(&str, &str)]) {
        let d = root.join(dir);
        fs::create_dir_all(&d).unwrap();
        for (name, content) in files {
            fs::write(d.join(name), content).unwrap();
        }
    }

    #[test]
    fn classifies_common_chips() {
        assert_eq!(classify_chip("coretemp"), "cpu");
        assert_eq!(classify_chip("k10temp"), "cpu");
        assert_eq!(classify_chip("amdgpu"), "gpu");
        assert_eq!(classify_chip("nvme"), "storage");
        assert_eq!(classify_chip("drivetemp"), "storage");
        assert_eq!(classify_chip("spd5118"), "memory");
        assert_eq!(classify_chip("nzxt-kraken3"), "cooling");
        assert_eq!(classify_chip("corsairpsu"), "psu");
        assert_eq!(classify_chip("nct6798"), "motherboard");
        assert_eq!(classify_chip("BAT0"), "battery");
        assert_eq!(classify_chip("somethingnew"), "other");
    }

    #[test]
    fn splits_attribute_names() {
        assert_eq!(split_attribute("temp3_input"), Some(("temp".into(), 3, "input".into())));
        assert_eq!(split_attribute("pwm2"), Some(("pwm".into(), 2, "".into())));
        assert_eq!(split_attribute("fan1_label"), Some(("fan".into(), 1, "label".into())));
        assert_eq!(split_attribute("name"), None);
    }

    #[test]
    fn reads_a_full_chip_with_every_kind_of_channel() {
        let root = std::env::temp_dir().join(format!("nitrux-hwmon-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fake_chip(&root, "hwmon/hwmon2", &[
            ("name", "nct6798\n"),
            ("temp1_input", "45500\n"), ("temp1_label", "SYSTIN\n"), ("temp1_max", "80000\n"), ("temp1_crit", "100000\n"),
            ("temp7_input", "-128000\n"),
            ("fan2_input", "1234\n"), ("fan2_label", "CPU_FAN\n"),
            ("pwm2", "128\n"),
            ("in0_input", "1200\n"),
            ("power1_average", "65000000\n"),
            ("curr1_input", "2500\n"), ("curr1_alarm", "1\n"),
        ]);
        let chips = read_all_sensors_in(&root.join("hwmon"), &root.join("thermal"));
        let _ = fs::remove_dir_all(&root);
        assert_eq!(chips.len(), 1);
        let c = &chips[0];
        assert_eq!((c.chip.as_str(), c.category.as_str()), ("nct6798", "motherboard"));
        let find = |kind: &str| c.readings.iter().find(|r| r.kind == kind).unwrap().clone();
        let t = find("temperature");
        assert_eq!((t.label.as_str(), t.value, t.max, t.critical), ("SYSTIN", 45.5, Some(80.0), Some(100.0)));
        assert_eq!(c.readings.iter().filter(|r| r.kind == "temperature").count(), 1, "the -128 °C placeholder is dropped");
        assert_eq!((find("fan").label.as_str(), find("fan").value), ("CPU_FAN", 1234.0));
        assert_eq!(find("pwm").value, 50.0);
        assert_eq!(find("voltage").value, 1.2);
        assert_eq!(find("power").value, 65.0);
        assert!(find("current").alarm);
    }

    #[test]
    fn falls_back_to_thermal_zones_when_hwmon_has_no_temperature() {
        let root = std::env::temp_dir().join(format!("nitrux-thermal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fake_chip(&root, "thermal/thermal_zone0", &[("type", "cpu-thermal\n"), ("temp", "52300\n")]);
        fs::create_dir_all(root.join("hwmon")).unwrap();
        let chips = read_all_sensors_in(&root.join("hwmon"), &root.join("thermal"));
        let _ = fs::remove_dir_all(&root);
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].category, "cpu");
        assert_eq!(chips[0].readings[0].value, 52.3);
    }

    #[test]
    fn reading_the_real_machine_never_panics() {
        let _ = get_all_sensors();
    }
}
