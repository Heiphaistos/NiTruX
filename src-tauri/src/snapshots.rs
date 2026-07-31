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
#[tauri::command]
pub fn list_snapshots() -> Result<Vec<Snapshot>, String> {
    let output = subprocess::run_with_timeout("timeshift", &["--list"], Duration::from_secs(10))?;
    Ok(output.lines().filter_map(parse_timeshift_line).collect())
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
}
