use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, Debug)]
pub struct Snapshot {
    pub id: String,
    pub date: String,
    /// What delete-snapshot takes: Timeshift's timestamp name, Snapper's number.
    pub handle: String,
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
    Some(Snapshot { id: id.to_string(), handle: date.clone(), date })
}

/// `snapper --csvout list --columns number,date,description`. Snapshot 0
/// is the running system, not a restore point.
pub fn parse_snapper_csv(csv: &str) -> Vec<Snapshot> {
    csv.lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.splitn(3, ',');
            let number = fields.next()?.trim();
            let date = fields.next()?.trim();
            let description = fields.next().unwrap_or("").trim().trim_matches('"');
            if number == "0" || number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let date = if description.is_empty() { date.to_string() } else { format!("{date} — {description}") };
            Some(Snapshot { id: number.to_string(), date, handle: number.to_string() })
        })
        .collect()
}

/// Snapper when it manages / (Omarchy, openSUSE, Fedora on btrfs) -- the
/// helper applies the same test -- otherwise Timeshift.
#[tauri::command]
pub fn get_snapshot_backend() -> String {
    let snapper_root = subprocess::binary_in_path("snapper")
        && subprocess::run_with_timeout("snapper", &["--csvout", "list-configs"], Duration::from_secs(10))
            .is_ok_and(|out| out.lines().any(|l| l.starts_with("root,")));
    if snapper_root { "snapper" } else { "timeshift" }.to_string()
}

/// Read-only: lists existing Btrfs/rsync snapshots via `timeshift --list`.
/// Never creates or deletes a snapshot itself -- those are separate,
/// privileged commands (`security_write::create_snapshot`/`delete_snapshot`).
/// Restoring a snapshot remains deliberately out of scope: unlike create/
/// delete, `timeshift --restore` needs a `--target` device and can leave
/// the system in a state that requires a reboot to recover from if it goes
/// wrong -- a different risk class from the two operations this module's
/// siblings already cover.
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
    if get_snapshot_backend() == "snapper" {
        // Snapper lists only for root: one password prompt, via the
        // read-only inventory action.
        let csv = subprocess::run_with_timeout(
            "pkexec",
            &["/usr/bin/nitrux-pkexec-inventory", "inventory", "snapper-list"],
            Duration::from_secs(60),
        )?;
        return Ok(parse_snapper_csv(&csv));
    }
    let (stdout, _stderr, code) = subprocess::run_capturing_exit_code("timeshift", &["--list"], Duration::from_secs(10))?;
    interpret_list_result(&stdout, code)
}

/// Puts one file back to its state in a snapshot (`number`: Snapper's
/// number or Timeshift's name); the helper keeps the current version as
/// `<file>.avant-restauration`.
#[tauri::command]
pub fn restore_snapshot_file(number: String, path: String) -> Result<String, String> {
    // Snapper numbers (digits) or Timeshift names (2026-10-12_10-00-00).
    if number.is_empty() || number == "0" || !number.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '_') {
        return Err(format!("instantané invalide : {number}"));
    }
    if !path.starts_with('/') || path.contains("..") || path.contains('\n') {
        return Err(format!("chemin invalide (absolu, sans « .. ») : {path}"));
    }
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-restore-snapshot-file", "restore-snapshot-file", &number, &path],
        Duration::from_secs(300),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_restore_refuses_bad_input_before_any_prompt() {
        assert!(restore_snapshot_file("0".into(), "/etc/hosts".into()).is_err());
        assert!(restore_snapshot_file("12;r".into(), "/etc/hosts".into()).is_err());
        assert!(restore_snapshot_file("2026-10-12_10;00".into(), "/etc/hosts".into()).is_err());
        assert!(restore_snapshot_file("12".into(), "etc/hosts".into()).is_err());
        assert!(restore_snapshot_file("12".into(), "/etc/../shadow".into()).is_err());
    }

    #[test]
    fn parses_snapper_csv_and_skips_the_running_system() {
        let csv = "number,date,description\n0,,current\n12,2026-10-01 10:00:00,omarchy 3.1\n13,2026-10-12 09:00:00,\"NiTruX manual snapshot\"\n";
        let snaps = parse_snapper_csv(csv);
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].handle, "12");
        assert_eq!(snaps[0].date, "2026-10-01 10:00:00 — omarchy 3.1");
        assert_eq!(snaps[1].date, "2026-10-12 09:00:00 — NiTruX manual snapshot");
    }

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
