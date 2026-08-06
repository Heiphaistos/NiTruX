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
    // `ufw` ships gettext .mo translations (confirmed live: the real
    // package's French catalog translates "Status: inactive" to "État :
    // inactif", the exact same class of locale-dependent parsing bug
    // already found and fixed for apt.rs's "[upgradable from:" text).
    // `get_firewall_status` is currently always invoked unprivileged (see
    // parse_ufw_output's own doc comment: that path hits an empty-stdout
    // permission error before ever reaching the "Status: ..." lines at
    // all), so this specific locale gap isn't reachable through this
    // app's current invocation pattern -- but forcing LC_ALL=C costs
    // nothing and closes the gap for e.g. the app being run as root,
    // mirroring the defensive-fix-even-if-not-currently-exploitable
    // precedent already established in this codebase.
    let output = subprocess::run_with_timeout_env("ufw", &["status"], &[("LC_ALL", "C")], Duration::from_secs(5))?;
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

    #[test]
    fn does_not_match_the_localized_status_text_this_command_relies_on_lc_all_c_to_avoid() {
        // ufw ships real gettext .mo translations -- extracted and queried
        // live from the actual French catalog in the real `ufw` package
        // (via `python3 gettext.GNUTranslations`, no live ufw invocation
        // needed since this only exercises the parser): under fr_FR,
        // "Status: inactive" becomes "État\u{a0}: inactif" (with the
        // French-typography non-breaking space before the colon). Without
        // get_firewall_status forcing LC_ALL=C, this line would hit the
        // catch-all "unexpected ufw response" error instead of being
        // recognized as a real, valid "inactive" status.
        let output = "État\u{a0}: inactif\n";
        assert!(parse_ufw_output(output).is_err(), "the parser must stay English-only; LC_ALL=C at the call site is the real fix");
    }
}
