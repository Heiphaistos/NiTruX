use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct FirewallStatus {
    pub active: bool,
    pub rules: Vec<String>,
}

/// Parses `ufw status` output. Rule lines are normalized to single-spaced
/// "To Action From" (the raw output is column-padded with variable
/// whitespace, which we collapse for a clean, consistent display string).
pub fn parse_ufw_output(output: &str) -> FirewallStatus {
    let active = output
        .lines()
        .next()
        .map(|l| l.trim() == "Status: active")
        .unwrap_or(false);

    if !active {
        return FirewallStatus { active: false, rules: Vec::new() };
    }

    let rules = output
        .lines()
        .skip_while(|l| !l.starts_with("--"))
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();

    FirewallStatus { active, rules }
}

#[tauri::command]
pub fn get_firewall_status() -> Result<FirewallStatus, String> {
    let output = subprocess::run_with_timeout("ufw", &["status"], Duration::from_secs(5))?;
    Ok(parse_ufw_output(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ufw_active_status_line() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert!(status.active);
        assert_eq!(status.rules.len(), 1);
        assert_eq!(status.rules[0], "22/tcp ALLOW Anywhere");
    }

    #[test]
    fn parses_ufw_inactive_status() {
        let output = "Status: inactive\n";
        let status = parse_ufw_output(output);
        assert!(!status.active);
        assert!(status.rules.is_empty());
    }

    #[test]
    fn parses_multiple_ufw_rules() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n80/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert_eq!(status.rules.len(), 2);
    }
}
