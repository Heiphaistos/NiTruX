use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct BluetoothStatus {
    pub adapter_present: bool,
    pub powered: bool,
    pub devices: Vec<BluetoothDevice>,
}

/// Parses `Powered: yes`/`Powered: no` out of `bluetoothctl show` output.
/// Any other line, or a missing `Powered:` line entirely (no adapter),
/// is treated as `false` -- an absent/unparseable status is never
/// reported as "on" by default.
pub fn parse_powered_status(show_output: &str) -> bool {
    show_output.lines().any(|l| l.trim() == "Powered: yes")
}

/// Parses one line of `bluetoothctl devices` output, e.g.
/// "Device 11:22:33:44:55:66 Sony WH-1000XM4" -> address + name. The name
/// may itself contain spaces, so only the first two whitespace-separated
/// tokens ("Device", the MAC) are consumed positionally; everything after
/// is the name verbatim.
pub fn parse_device_line(line: &str) -> Option<BluetoothDevice> {
    let rest = line.strip_prefix("Device ")?;
    let (address, name) = rest.split_once(' ')?;
    if address.is_empty() || name.is_empty() {
        return None;
    }
    Some(BluetoothDevice { address: address.to_string(), name: name.to_string() })
}

#[tauri::command]
pub fn get_bluetooth_status() -> BluetoothStatus {
    let show_output = subprocess::run_with_timeout("bluetoothctl", &["show"], Duration::from_secs(5));
    let adapter_present = show_output.is_ok();
    let powered = show_output.as_deref().map(parse_powered_status).unwrap_or(false);

    let devices = subprocess::run_with_timeout("bluetoothctl", &["devices"], Duration::from_secs(5))
        .map(|output| output.lines().filter_map(parse_device_line).collect())
        .unwrap_or_default();

    BluetoothStatus { adapter_present, powered, devices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_powered_yes() {
        let output = "Controller AA:BB:CC:DD:EE:FF (public)\n\tName: dev-laptop\n\tPowered: yes\n\tDiscoverable: no\n";
        assert!(parse_powered_status(output));
    }

    #[test]
    fn parses_powered_no() {
        let output = "Controller AA:BB:CC:DD:EE:FF (public)\n\tPowered: no\n";
        assert!(!parse_powered_status(output));
    }

    #[test]
    fn treats_missing_powered_line_as_false() {
        assert!(!parse_powered_status("Controller AA:BB:CC:DD:EE:FF (public)\n\tName: dev-laptop\n"));
    }

    #[test]
    fn parses_a_device_line_with_a_multi_word_name() {
        let line = "Device 11:22:33:44:55:66 Sony WH-1000XM4";
        let device = parse_device_line(line).expect("should parse");
        assert_eq!(device.address, "11:22:33:44:55:66");
        assert_eq!(device.name, "Sony WH-1000XM4");
    }

    #[test]
    fn rejects_a_line_not_starting_with_device_prefix() {
        assert!(parse_device_line("Controller AA:BB:CC:DD:EE:FF (public)").is_none());
    }
}
