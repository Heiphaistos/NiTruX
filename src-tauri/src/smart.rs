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

/// Extracts smartctl's actual failure explanation from its stdout, e.g.
/// "Smartctl open device: /dev/sdd failed: Permission denied". Confirmed
/// live (extracting the real `smartctl` binary via `apt-get
/// download`/`dpkg-deb -x`, no root needed, run unprivileged against a
/// real block device in this dev environment) that on a communication
/// failure smartctl prints its actual, actionable reason to STDOUT
/// (alongside its version/copyright banner) with an EMPTY stderr -- the
/// same "real explanation lives on stdout, not stderr" pattern already
/// found and fixed for `timeshift` in `snapshots.rs`. Filters out the
/// banner lines (version string, copyright) rather than assuming a fixed
/// line position, since smartctl's banner content varies by build.
fn extract_failure_reason(output: &str) -> Option<String> {
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("smartctl ") && !l.starts_with("Copyright"))
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// `smartctl`'s exit status is a BITMASK, not a simple 0/success ==
/// everything-else/error scheme (confirmed against the smartctl(8) man
/// page): bit 0 (1) = command line didn't parse, bit 1 (2) = device open
/// failed, bit 2 (4) = a SMART/ATA command failed, bit 3 (8) = health
/// check returned "DISK FAILING", bit 4 (16) = prefail attributes at/below
/// threshold, bit 5 (32) = an attribute was below threshold in the past.
/// Bits 3-5 report REAL health data through a non-zero exit code -- most
/// critically, bit 3 is exactly the "the disk is failing" case this
/// command exists to surface. Only bits 0-2 (device/command genuinely
/// unreachable) leave no usable health line to parse.
fn interpret_smartctl_result(output: &str, code: i32) -> Result<Option<String>, String> {
    const COMMUNICATION_FAILURE_BITS: i32 = 0b111; // bits 0-2
    if code & COMMUNICATION_FAILURE_BITS != 0 {
        return Err(match extract_failure_reason(output) {
            Some(reason) => format!("smartctl n'a pas pu interroger le périphérique (code {code}) : {reason}"),
            None => format!("smartctl n'a pas pu interroger le périphérique (code {code})"),
        });
    }
    Ok(parse_health_line(output))
}

/// Queries SMART health for `device` (e.g. "/dev/sda"). `smartctl` commonly
/// requires root to access the raw device — a permission-denied failure is
/// surfaced as a normal `Err`, not a crash. This is a real, expected
/// limitation on most systems (see design spec §5.1's note on `dmidecode`
/// having the same root requirement), not something this task works around.
#[tauri::command]
pub fn get_smart_status(device: String) -> Result<SmartStatus, String> {
    let (output, code) = subprocess::run_capturing_exit_code("smartctl", &["-H", &device], Duration::from_secs(15))?;
    Ok(SmartStatus {
        device,
        health: interpret_smartctl_result(&output, code)?,
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

    #[test]
    fn a_failing_disk_health_line_is_still_returned_despite_the_nonzero_exit_code() {
        // Exit code 8 == bit 3 alone == "SMART status check returned DISK
        // FAILING" per smartctl(8) -- real, critical health data, not a
        // communication failure. Previously this exact case was discarded
        // as a hard Err by run_with_timeout (which drops stdout on any
        // non-zero exit), hiding the single most important result this
        // command exists to surface.
        let output = "SMART overall-health self-assessment test result: FAILED!\n";
        let result = interpret_smartctl_result(output, 8).expect("bit 3 alone should not be an error");
        assert_eq!(result, Some("FAILED!".to_string()));
    }

    #[test]
    fn a_passing_disk_with_a_past_threshold_attribute_is_still_returned() {
        // Exit code 32 == bit 5 == "DISK OK but some attribute was below
        // threshold in the past" -- also real health data, not a failure.
        let output = "SMART overall-health self-assessment test result: PASSED\n";
        let result = interpret_smartctl_result(output, 32).expect("bit 5 alone should not be an error");
        assert_eq!(result, Some("PASSED".to_string()));
    }

    #[test]
    fn device_open_failure_is_still_a_real_error() {
        // Exit code 2 == bit 1 == device open failed -- no reliable health
        // line exists in this case, this must remain an error.
        let err = interpret_smartctl_result("", 2).expect_err("bit 1 should be a real error");
        assert!(err.contains('2'), "error should mention the exit code: {err}");
    }

    #[test]
    fn device_open_failure_surfaces_smartctls_real_explanation_instead_of_losing_it() {
        // Regression test using the EXACT raw output captured live from a
        // real smartctl binary run unprivileged against a real block
        // device in this dev environment (permission denied). Before this
        // fix, the banner+explanation on stdout was entirely discarded,
        // leaving the user with a bare "(code 2)" and no indication that
        // running as admin would fix it.
        let output = "smartctl 7.2 2020-12-30 r5155 [x86_64-linux-6.18.33.2-microsoft-standard-WSL2] (local build)\nCopyright (C) 2002-20, Bruce Allen, Christian Franke, www.smartmontools.org\n\nSmartctl open device: /dev/sdd failed: Permission denied\n";
        let err = interpret_smartctl_result(output, 2).expect_err("bit 1 should be a real error");
        assert!(err.contains("Permission denied"), "error should preserve smartctl's real explanation: {err}");
        assert!(!err.contains("Copyright"), "banner lines should be filtered out: {err}");
    }

    #[test]
    fn extract_failure_reason_filters_out_the_version_and_copyright_banner() {
        let output = "smartctl 7.2 2020-12-30 r5155 (local build)\nCopyright (C) 2002-20, Bruce Allen\n\nSmartctl open device: /dev/sda failed: No such device\n";
        assert_eq!(extract_failure_reason(output).as_deref(), Some("Smartctl open device: /dev/sda failed: No such device"));
    }

    #[test]
    fn extract_failure_reason_returns_none_for_purely_banner_output() {
        let output = "smartctl 7.2 2020-12-30 r5155 (local build)\nCopyright (C) 2002-20, Bruce Allen\n";
        assert_eq!(extract_failure_reason(output), None);
    }

    #[test]
    fn clean_exit_parses_normally() {
        let output = "SMART overall-health self-assessment test result: PASSED\n";
        let result = interpret_smartctl_result(output, 0).expect("exit 0 should be Ok");
        assert_eq!(result, Some("PASSED".to_string()));
    }
}
