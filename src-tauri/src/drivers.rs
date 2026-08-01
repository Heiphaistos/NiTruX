use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct DeviceDriver {
    pub slot: String,
    pub description: String,
    /// `None` when no kernel driver is currently bound to this device —
    /// a real, honest state (not every PCI device has one), not an error.
    pub driver: Option<String>,
}

/// Parses `lspci -k` output into one entry per device block. A device
/// block starts with an unindented line (`<slot> <class>: <description>`)
/// and is followed by zero or more tab-indented detail lines, of which
/// only `Kernel driver in use: <name>` is extracted here — `Subsystem:`
/// and `Kernel modules:` lines are ignored (the *bound* driver is what
/// matters for "what's actually running", not every module that
/// could theoretically bind). Lines that don't parse as a device header
/// (no `slot description: text` shape) are skipped rather than causing
/// the whole parse to fail or misattributing trailing indented lines to
/// the wrong device — mirrors `hardware::parse_lspci_line`'s existing
/// tolerance for malformed lines.
pub fn parse_lspci_k_output(output: &str) -> Vec<DeviceDriver> {
    let mut devices = Vec::new();
    for line in output.lines() {
        if !line.starts_with(char::is_whitespace) {
            if let Some((slot, rest)) = line.split_once(' ') {
                if let Some((_class, description)) = rest.split_once(": ") {
                    devices.push(DeviceDriver {
                        slot: slot.to_string(),
                        description: description.to_string(),
                        driver: None,
                    });
                    continue;
                }
            }
        } else if let Some(current) = devices.last_mut() {
            let trimmed = line.trim_start();
            if let Some(driver_name) = trimmed.strip_prefix("Kernel driver in use: ") {
                current.driver = Some(driver_name.to_string());
            }
        }
    }
    devices
}

fn run_lspci_k() -> Result<Vec<DeviceDriver>, String> {
    let output = subprocess::run_with_timeout("lspci", &["-k"], Duration::from_secs(5))
        .map_err(|e| format!("{e} (paquet requis : pciutils)"))?;
    Ok(parse_lspci_k_output(&output))
}

#[derive(Serialize, Clone)]
pub struct DriverSnapshot {
    pub loaded_modules: Vec<String>,
    pub gpu_driver: String,
    pub devices: Vec<DeviceDriver>,
}

pub fn parse_lsmod_line(line: &str) -> Option<String> {
    let name = line.split_whitespace().next()?;
    if name.eq_ignore_ascii_case("module") {
        return None; // header row
    }
    Some(name.to_string())
}

pub fn detect_gpu_driver(modules: &[String]) -> String {
    if modules.iter().any(|m| m == "nvidia") {
        "nvidia (propriétaire)".to_string()
    } else if modules.iter().any(|m| m == "nouveau") {
        "nouveau (open-source)".to_string()
    } else if modules.iter().any(|m| m == "amdgpu") {
        "amdgpu (open-source)".to_string()
    } else if modules.iter().any(|m| m == "i915") {
        "i915 (Intel, open-source)".to_string()
    } else {
        "inconnu".to_string()
    }
}

fn run_lsmod() -> Result<Vec<String>, String> {
    let output = subprocess::run_with_timeout("lsmod", &[], Duration::from_secs(5))?;
    Ok(output.lines().filter_map(parse_lsmod_line).collect())
}

#[tauri::command]
pub fn get_driver_snapshot() -> Result<DriverSnapshot, String> {
    let loaded_modules = run_lsmod()?;
    let gpu_driver = detect_gpu_driver(&loaded_modules);
    let devices = run_lspci_k().unwrap_or_default();
    Ok(DriverSnapshot { loaded_modules, gpu_driver, devices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nvidia_driver_from_module_list() {
        let modules = vec!["nvidia".to_string(), "snd_hda_intel".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nvidia (propriétaire)");
    }

    #[test]
    fn detects_nouveau_driver_from_module_list() {
        let modules = vec!["nouveau".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nouveau (open-source)");
    }

    #[test]
    fn detects_amdgpu_driver_from_module_list() {
        let modules = vec!["amdgpu".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "amdgpu (open-source)");
    }

    #[test]
    fn falls_back_to_unknown_when_no_gpu_module_present() {
        let modules = vec!["ext4".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "inconnu");
    }

    #[test]
    fn detects_i915_driver_from_module_list() {
        let modules = vec!["i915".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "i915 (Intel, open-source)");
    }

    #[test]
    fn parses_lsmod_line_into_module_name() {
        let line = "nvidia               56655872  42";
        assert_eq!(parse_lsmod_line(line), Some("nvidia".to_string()));
    }

    #[test]
    fn skips_lsmod_header_row() {
        let line = "Module                  Size  Used by";
        assert_eq!(parse_lsmod_line(line), None);
    }

    #[test]
    fn parses_a_device_block_with_kernel_driver_in_use() {
        let output = "5582:00:00.0 SCSI storage controller: Red Hat, Inc. Virtio console (rev 01)\n\tSubsystem: Red Hat, Inc. Virtio console\n\tKernel driver in use: virtio-pci\n\tKernel modules: virtio_pci\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].slot, "5582:00:00.0");
        assert_eq!(devices[0].description, "Red Hat, Inc. Virtio console (rev 01)");
        assert_eq!(devices[0].driver.as_deref(), Some("virtio-pci"));
    }

    #[test]
    fn parses_multiple_device_blocks() {
        let output = "5582:00:00.0 SCSI storage controller: Red Hat, Inc. Virtio console (rev 01)\n\tKernel driver in use: virtio-pci\n64f4:00:00.0 System peripheral: Red Hat, Inc. Virtio file system (rev 01)\n\tKernel driver in use: virtio-pci\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[1].slot, "64f4:00:00.0");
    }

    #[test]
    fn maps_missing_kernel_driver_line_to_none_rather_than_dropping_the_device() {
        let output = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620\n\tSubsystem: Some Vendor Device\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].driver, None);
    }

    #[test]
    fn skips_malformed_device_header_lines_without_dropping_valid_siblings() {
        let output = "not a valid header line\n\tKernel driver in use: bogus\n00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620\n\tKernel driver in use: i915\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].slot, "00:02.0");
        assert_eq!(devices[0].driver.as_deref(), Some("i915"));
    }
}
