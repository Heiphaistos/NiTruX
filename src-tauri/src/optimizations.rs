use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct OptimizationSnapshot {
    pub enabled_services: Vec<String>,
    pub swappiness: Option<u8>,
    pub zram_active: bool,
    pub fstrim_timer_enabled: bool,
}

/// Parses one line of `systemctl list-unit-files --type=service
/// --state=enabled --no-legend` output into just the unit name (first
/// whitespace-separated field), e.g. "accounts-daemon.service             enabled enabled"
/// -> "accounts-daemon.service". Blank lines are skipped.
pub fn parse_enabled_service_line(line: &str) -> Option<String> {
    let name = line.split_whitespace().next()?;
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

/// Parses `/proc/sys/vm/swappiness` content (a single integer 0-100 on its
/// own line) -- mirrors `sensors::parse_capacity`'s exact pattern for the
/// same reason: a single trimmed-and-parsed `/proc`/`/sys` integer file.
pub fn parse_swappiness(content: &str) -> Option<u8> {
    content.trim().parse::<u8>().ok()
}

/// True if any active swap device (from `/proc/swaps` content) is backed
/// by a zram device. Skips the header line (`Filename Type Size Used
/// Priority`) that `/proc/swaps` always starts with.
pub fn detect_zram_active(proc_swaps_content: &str) -> bool {
    proc_swaps_content.lines().skip(1).any(|line| line.contains("zram"))
}

fn run_optimization_diagnostics() -> OptimizationSnapshot {
    let enabled_services = subprocess::run_with_timeout(
        "systemctl",
        &["list-unit-files", "--type=service", "--state=enabled", "--no-legend"],
        Duration::from_secs(5),
    )
    .map(|output| output.lines().filter_map(parse_enabled_service_line).collect())
    .unwrap_or_default();

    let swappiness = std::fs::read_to_string("/proc/sys/vm/swappiness")
        .ok()
        .and_then(|content| parse_swappiness(&content));

    let zram_active = std::fs::read_to_string("/proc/swaps")
        .map(|content| detect_zram_active(&content))
        .unwrap_or(false);

    let fstrim_timer_enabled = subprocess::run_capturing_exit_code(
        "systemctl",
        &["is-enabled", "fstrim.timer"],
        Duration::from_secs(5),
    )
    .map(|(stdout, _code)| stdout.trim() == "enabled")
    .unwrap_or(false);

    OptimizationSnapshot { enabled_services, swappiness, zram_active, fstrim_timer_enabled }
}

#[tauri::command]
pub fn get_optimization_snapshot() -> OptimizationSnapshot {
    run_optimization_diagnostics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enabled_service_line_into_unit_name() {
        let line = "accounts-daemon.service             enabled enabled";
        assert_eq!(parse_enabled_service_line(line), Some("accounts-daemon.service".to_string()));
    }

    #[test]
    fn skips_blank_lines_in_service_list() {
        assert_eq!(parse_enabled_service_line(""), None);
        assert_eq!(parse_enabled_service_line("   "), None);
    }

    #[test]
    fn parses_valid_swappiness_value() {
        assert_eq!(parse_swappiness("60\n"), Some(60));
        assert_eq!(parse_swappiness("0"), Some(0));
    }

    #[test]
    fn rejects_malformed_swappiness_content() {
        assert_eq!(parse_swappiness("not a number"), None);
        assert_eq!(parse_swappiness(""), None);
    }

    #[test]
    fn detects_zram_swap_device_in_proc_swaps() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/dev/zram0                              partition\t8388604\t\t0\t\t100\n";
        assert!(detect_zram_active(content));
    }

    #[test]
    fn returns_false_when_no_swap_device_is_zram() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/dev/sda3                               partition\t6873084\t\t524\t\t-2\n";
        assert!(!detect_zram_active(content));
    }

    #[test]
    fn returns_false_when_no_swap_devices_at_all() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n";
        assert!(!detect_zram_active(content));
    }
}
