use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct SmartStatus {
    pub device: String,
    pub health: Option<String>,
}

/// Extracts the value after "SMART overall-health self-assessment test
/// result:" from `smartctl -H` output, e.g. "PASSED" or "FAILED!".
pub fn parse_health_line(output: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains("SMART overall-health self-assessment test result:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
}

/// Queries SMART health for `device` (e.g. "/dev/sda"). `smartctl` commonly
/// requires root to access the raw device — a permission-denied failure is
/// surfaced as a normal `Err`, not a crash. This is a real, expected
/// limitation on most systems (see design spec §5.1's note on `dmidecode`
/// having the same root requirement), not something this task works around.
#[tauri::command]
pub fn get_smart_status(device: String) -> Result<SmartStatus, String> {
    let output = subprocess::run_with_timeout("smartctl", &["-H", &device], Duration::from_secs(15))?;
    Ok(SmartStatus {
        device,
        health: parse_health_line(&output),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smart_overall_health_line() {
        let output = "SMART overall-health self-assessment test result: PASSED\n";
        assert_eq!(parse_health_line(output), Some("PASSED".to_string()));
    }

    #[test]
    fn returns_none_when_health_line_is_absent() {
        let output = "smartctl 7.2 2020-12-30 r5155\nSome other output\n";
        assert_eq!(parse_health_line(output), None);
    }

    #[test]
    fn handles_failed_health_status() {
        let output = "SMART overall-health self-assessment test result: FAILED!\n";
        assert_eq!(parse_health_line(output), Some("FAILED!".to_string()));
    }
}
