use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct PciDevice {
    pub slot: String,
    pub class: String,
    pub description: String,
}

pub fn parse_lspci_line(line: &str) -> Option<PciDevice> {
    let (slot, rest) = line.split_once(' ')?;
    let (class, description) = rest.split_once(": ")?;
    Some(PciDevice {
        slot: slot.to_string(),
        class: class.to_string(),
        description: description.to_string(),
    })
}

fn run_lspci() -> Vec<PciDevice> {
    let output = match Command::new("lspci").output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().filter_map(parse_lspci_line).collect()
}

#[tauri::command]
pub fn get_pci_devices() -> Vec<PciDevice> {
    run_lspci()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lspci_line_into_device() {
        let line = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620";
        let device = parse_lspci_line(line).expect("should parse");
        assert_eq!(device.slot, "00:02.0");
        assert_eq!(device.class, "VGA compatible controller");
        assert_eq!(device.description, "Intel Corporation UHD Graphics 620");
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_lspci_line("not a valid line").is_none());
    }
}
