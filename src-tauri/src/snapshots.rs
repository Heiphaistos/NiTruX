use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, Debug)]
pub struct Snapshot {
    pub id: String,
    pub date: String,
}

/// Parses one line of `timeshift --list` output, e.g.:
/// "1 > 2026-07-30_23-00-01                     O"
/// (id, optional ">" marker for the current default, timestamp, tags).
/// Timeshift's list format varies slightly by version in whether the ">"
/// marker is present; this handles both by looking for the first
/// timestamp-shaped token (YYYY-MM-DD_HH-MM-SS) rather than a fixed
/// column position.
pub fn parse_timeshift_line(line: &str) -> Option<Snapshot> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.is_empty() {
        return None;
    }
    let id = fields[0];
    if !id.chars().next()?.is_ascii_digit() {
        return None; // header row or other non-data line
    }
    let date = fields
        .iter()
        .find(|f| f.len() == 19 && f.chars().nth(4) == Some('-'))?
        .to_string();
    Some(Snapshot { id: id.to_string(), date })
}

/// Read-only: lists existing Btrfs/rsync snapshots via `timeshift --list`.
/// Never creates, restores, or deletes a snapshot — that is out of scope for
/// this plan, same as every other write-capable operation deferred across
/// prior phases.
///
/// Uses `run_capturing_exit_code` rather than `run_with_timeout`: confirmed
/// live (extracting the real `timeshift` binary and running it unprivileged
/// on this dev machine, since it isn't installed here) that timeshift exits
/// non-zero (1) when it needs admin access, and -- unlike most tools in this
/// codebase -- prints its actual, actionable explanation ("Application
/// needs admin access. Please run the application as admin (using 'sudo' or
/// 'su')") to STDOUT, with an EMPTY stderr. `run_with_timeout` only
/// includes stderr in its error message, so that real explanation was
/// being silently discarded, leaving the user with a bare, uninformative
/// "timeshift a échoué (code 1) : " (nothing after the colon).
fn interpret_list_result(stdout: &str, code: i32) -> Result<Vec<Snapshot>, String> {
    if code != 0 {
        return Err(format!("timeshift a échoué (code {code}) : {}", stdout.trim()));
    }
    Ok(stdout.lines().filter_map(parse_timeshift_line).collect())
}

#[tauri::command]
pub fn list_snapshots() -> Result<Vec<Snapshot>, String> {
    let (stdout, code) = subprocess::run_capturing_exit_code("timeshift", &["--list"], Duration::from_secs(10))?;
    interpret_list_result(&stdout, code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timeshift_list_line() {
        let line = "1 > 2026-07-30_23-00-01                     O";
        let snap = parse_timeshift_line(line).expect("should parse");
        assert_eq!(snap.id, "1");
        assert_eq!(snap.date, "2026-07-30_23-00-01");
    }

    #[test]
    fn skips_timeshift_header_line() {
        assert!(parse_timeshift_line("Num     Name                               Tags").is_none());
    }

    #[test]
    fn skips_empty_lines() {
        assert!(parse_timeshift_line("").is_none());
        assert!(parse_timeshift_line("   ").is_none());
    }

    #[test]
    fn zero_exit_parses_real_snapshot_lines() {
        let stdout = "Num     Name                               Tags\n1 > 2026-07-30_23-00-01                     O\n";
        let result = interpret_list_result(stdout, 0).expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "1");
    }

    #[test]
    fn nonzero_exit_surfaces_the_real_explanation_from_stdout_instead_of_losing_it() {
        // Regression test: confirmed live that timeshift prints its actual
        // explanation to stdout (not stderr) on a permission failure --
        // before this fix, run_with_timeout only captured stderr, so this
        // explanation was silently discarded and the user saw a bare,
        // empty "timeshift a échoué (code 1) : " with no reason at all.
        let stdout = "Application needs admin access.\nPlease run the application as admin (using 'sudo' or 'su')\n";
        let err = interpret_list_result(stdout, 1).expect_err("should fail");
        assert!(err.contains("admin access"), "error should preserve timeshift's real explanation: {err}");
        assert!(err.contains("code 1"));
    }
}
