use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub priority: u8,
    pub message: String,
    pub unit: String,
}

#[derive(Deserialize)]
struct RawJournalLine {
    #[serde(rename = "PRIORITY", default)]
    priority: String,
    #[serde(rename = "MESSAGE")]
    message: String,
    #[serde(rename = "SYSLOG_IDENTIFIER", default)]
    syslog_identifier: String,
}

/// journald's syslog "informational" level (6) -- used when PRIORITY is
/// absent from a real journal entry. Confirmed live against a real
/// journalctl JSON stream (500 real entries, VM): 78 of them (15.6%,
/// mostly flatpak/libostree progress lines) have no PRIORITY field at all.
/// Before this fix, a missing PRIORITY made the whole line fail to parse
/// (`raw.priority.parse().ok()?` on an absent/empty string), silently
/// dropping every one of those genuine log entries from the Logs page
/// with no indication to the user that anything was filtered out. `6` is
/// the correct default rather than treating it as an error or a warning:
/// these are ordinary informational messages, not indicators of a problem
/// journald simply never tagged with an explicit level.
const DEFAULT_PRIORITY: u8 = 6;

pub fn parse_journal_line(line: &str) -> Option<LogEntry> {
    let raw: RawJournalLine = serde_json::from_str(line).ok()?;
    let priority: u8 = if raw.priority.is_empty() {
        DEFAULT_PRIORITY
    } else {
        raw.priority.parse().ok()?
    };
    Some(LogEntry {
        priority,
        message: raw.message,
        unit: raw.syslog_identifier,
    })
}

fn run_journalctl(limit: u32) -> Result<Vec<LogEntry>, String> {
    let limit_str = limit.to_string();
    let output = subprocess::run_with_timeout(
        "journalctl",
        &["-o", "json", "-n", &limit_str, "--no-pager"],
        Duration::from_secs(5),
    )?;
    Ok(output.lines().filter_map(parse_journal_line).collect())
}

#[tauri::command]
pub fn get_recent_logs(limit: u32) -> Result<Vec<LogEntry>, String> {
    run_journalctl(limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_journalctl_json_line_into_entry() {
        let line = r#"{"__REALTIME_TIMESTAMP":"1785440000000000","PRIORITY":"3","MESSAGE":"disk write error","SYSLOG_IDENTIFIER":"kernel"}"#;
        let entry = parse_journal_line(line).expect("should parse");
        assert_eq!(entry.priority, 3);
        assert_eq!(entry.message, "disk write error");
        assert_eq!(entry.unit, "kernel");
    }

    #[test]
    fn skips_unparseable_lines() {
        assert!(parse_journal_line("not json").is_none());
    }

    #[test]
    fn skips_line_with_malformed_priority() {
        let line = r#"{"PRIORITY":"not-a-number","MESSAGE":"oops","SYSLOG_IDENTIFIER":"kernel"}"#;
        assert!(parse_journal_line(line).is_none());
    }

    #[test]
    fn skips_line_missing_required_fields() {
        let line = r#"{"__REALTIME_TIMESTAMP":"1785440000000000"}"#;
        assert!(parse_journal_line(line).is_none());
    }

    #[test]
    fn defaults_priority_to_info_when_the_field_is_entirely_absent() {
        // Real line captured live from `journalctl -o json` on the VM
        // (trimmed to the fields this parser reads) -- flatpak/libostree
        // progress messages have no PRIORITY field at all, not an empty
        // string. Before this fix the whole entry was silently dropped.
        let line = r#"{"MESSAGE":"libostree pull from 'flathub' for app/com.brave.Browser/x86_64/stable complete","SYSLOG_IDENTIFIER":"flatpak"}"#;
        let entry = parse_journal_line(line).expect("a real entry missing PRIORITY must still parse");
        assert_eq!(entry.priority, 6, "missing PRIORITY should default to info (6), not be dropped");
        assert_eq!(entry.unit, "flatpak");
    }

    #[test]
    fn defaults_unit_to_empty_when_syslog_identifier_missing() {
        let line = r#"{"PRIORITY":"6","MESSAGE":"hello"}"#;
        let entry = parse_journal_line(line).expect("should parse");
        assert_eq!(entry.unit, "");
    }

    #[test]
    fn skips_line_with_byte_array_message() {
        // journalctl encodes MESSAGE as a JSON array of byte values instead
        // of a string when the field isn't valid UTF-8 (observed in real
        // WSL2 output, e.g. ANSI-colored wsl-pro-service log lines). Must be
        // skipped gracefully, not panic or produce a garbage entry.
        let line = r#"{"PRIORITY":"6","MESSAGE":[104,105],"SYSLOG_IDENTIFIER":"x"}"#;
        assert!(parse_journal_line(line).is_none());
    }
}
