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
///
/// `ufw status` run unprivileged (NiTruX's normal, by-design mode -- see
/// terminal.rs's module doc comment) exits 0 with EMPTY stdout and
/// "ERROR: You need to be root to run this script" on stderr, which
/// `run_with_timeout` discards on a successful exit code. Naively
/// defaulting anything other than the exact "Status: active" first line to
/// "inactive" (the previous behavior) made this permission failure
/// indistinguishable from a genuinely disabled firewall -- reproduced
/// live on this dev machine's real ufw binary, confirmed exit 0 + blank
/// stdout. Now only the two real, known-good first lines are accepted;
/// anything else (blank, or any other unexpected ufw output) is a real
/// error surfaced to the user instead of a silent, wrong "inactive".
pub fn parse_ufw_output(output: &str) -> Result<FirewallStatus, String> {
    match output.lines().next().map(str::trim) {
        Some("Status: inactive") => Ok(FirewallStatus { active: false, rules: Vec::new() }),
        Some("Status: active") => {
            let rules = output
                .lines()
                .skip_while(|l| !l.starts_with("--"))
                .skip(1)
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
                .collect();
            Ok(FirewallStatus { active: true, rules })
        }
        _ if output.trim().is_empty() => {
            Err("impossible de lire l'état d'ufw : réponse vide (droits administrateur probablement requis)".to_string())
        }
        _ => Err(format!("réponse inattendue d'ufw status : {}", output.trim())),
    }
}

#[tauri::command]
pub fn get_firewall_status() -> Result<FirewallStatus, String> {
    let output = subprocess::run_with_timeout("ufw", &["status"], Duration::from_secs(5))?;
    parse_ufw_output(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ufw_active_status_line() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output).expect("should parse");
        assert!(status.active);
        assert_eq!(status.rules.len(), 1);
        assert_eq!(status.rules[0], "22/tcp ALLOW Anywhere");
    }

    #[test]
    fn parses_ufw_inactive_status() {
        let output = "Status: inactive\n";
        let status = parse_ufw_output(output).expect("should parse");
        assert!(!status.active);
        assert!(status.rules.is_empty());
    }

    #[test]
    fn parses_multiple_ufw_rules() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n80/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output).expect("should parse");
        assert_eq!(status.rules.len(), 2);
    }

    // Regression guard for the actual bug: reproduced live on this dev
    // machine's real `ufw` binary run unprivileged -- exit code 0, empty
    // stdout, "ERROR: You need to be root to run this script" on stderr
    // (discarded by run_with_timeout on a successful exit code). The old
    // code silently reported this as a genuinely disabled firewall.
    #[test]
    fn empty_output_is_a_real_error_not_silently_reported_as_inactive() {
        let result = parse_ufw_output("");
        assert!(result.is_err(), "empty ufw output (the real unprivileged-run symptom) must be an error, not a false 'inactive'");
    }

    #[test]
    fn unrecognized_first_line_is_a_real_error_not_silently_reported_as_inactive() {
        let result = parse_ufw_output("ERROR: You need to be root to run this script\n");
        assert!(result.is_err());
    }
}
