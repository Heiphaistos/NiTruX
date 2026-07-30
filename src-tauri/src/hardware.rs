use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

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

fn run_lspci() -> Result<Vec<PciDevice>, String> {
    let stdout = subprocess::run_with_timeout("lspci", &[], Duration::from_secs(5))
        .map_err(|e| format!("{e} (paquet requis : pciutils)"))?;
    Ok(stdout.lines().filter_map(parse_lspci_line).collect())
}

#[tauri::command]
pub fn get_pci_devices() -> Result<Vec<PciDevice>, String> {
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

    #[test]
    fn drops_line_with_missing_description_separator() {
        // A trailing colon with nothing after it has no ": " (colon+space)
        // separator to split on, so this is treated as malformed and
        // dropped rather than surfaced as a device with an empty
        // description — an intentional choice, not an oversight.
        assert!(parse_lspci_line("00:02.0 Some class:").is_none());
    }
}
