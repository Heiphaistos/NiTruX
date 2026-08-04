use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct UpdateHistoryEntry {
    pub start_date: String,
    pub commandline: String,
    /// Combined summary of Install:/Upgrade:/Remove: lines for this block,
    /// whichever were present -- not split further per-package, this is a
    /// diagnostic overview, not a full per-package audit trail.
    pub summary: String,
}

/// Parses `/var/log/apt/history.log` content, which is a sequence of
/// blank-line-separated blocks each starting with "Start-Date:" and
/// containing "Commandline:" plus one or more of the six action headers
/// apt itself writes (confirmed against `libapt-pkg`'s own symbol strings:
/// `Install`, `Upgrade`, `Reinstall`, `Downgrade`, `Remove`, `Purge` are
/// all real `APT::StateChanges` categories, not just the three this parser
/// originally recognized), ending with "End-Date:". Malformed or
/// incomplete blocks (missing Start-Date or Commandline) are skipped
/// rather than producing a partial/garbage entry.
pub fn parse_apt_history(content: &str) -> Vec<UpdateHistoryEntry> {
    const SUMMARY_PREFIXES: &[&str] = &["Install: ", "Upgrade: ", "Reinstall: ", "Downgrade: ", "Remove: ", "Purge: "];
    let mut entries = Vec::new();
    for block in content.split("\n\n") {
        let mut start_date = None;
        let mut commandline = None;
        let mut summary_parts = Vec::new();
        for line in block.lines() {
            if let Some(v) = line.strip_prefix("Start-Date: ") {
                start_date = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("Commandline: ") {
                commandline = Some(v.to_string());
            } else if SUMMARY_PREFIXES.iter().any(|p| line.starts_with(p)) {
                summary_parts.push(line.to_string());
            }
        }
        if let (Some(start_date), Some(commandline)) = (start_date, commandline) {
            entries.push(UpdateHistoryEntry { start_date, commandline, summary: summary_parts.join(" | ") });
        }
    }
    entries
}

#[tauri::command]
pub fn get_update_history() -> Result<Vec<UpdateHistoryEntry>, String> {
    if !std::path::Path::new("/var/log/apt/history.log").exists() {
        return Err("historique non disponible pour ce gestionnaire de paquets".to_string());
    }
    let content = std::fs::read_to_string("/var/log/apt/history.log")
        .map_err(|e| format!("impossible de lire l'historique : {e}"))?;
    let mut entries = parse_apt_history(&content);
    entries.reverse(); // most recent first
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_apt_history_block() {
        let content = "Start-Date: 2026-08-02  01:48:01\nCommandline: apt-get install -y flatpak\nInstall: flatpak:amd64 (1.16.6-1~deb13u1)\nEnd-Date: 2026-08-02  01:48:05\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_date, "2026-08-02  01:48:01");
        assert_eq!(entries[0].commandline, "apt-get install -y flatpak");
        assert!(entries[0].summary.contains("flatpak"));
    }

    #[test]
    fn parses_multiple_blocks_separated_by_blank_lines() {
        let content = "Start-Date: 2026-08-01\nCommandline: apt-get update\nEnd-Date: 2026-08-01\n\nStart-Date: 2026-08-02\nCommandline: apt-get install -y snapd\nInstall: snapd:amd64 (2.68.3)\nEnd-Date: 2026-08-02\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn skips_a_block_missing_commandline() {
        let content = "Start-Date: 2026-08-01\nEnd-Date: 2026-08-01\n";
        let entries = parse_apt_history(content);
        assert!(entries.is_empty());
    }

    // Regression guard for the actual bug: `libapt-pkg`'s own APT::StateChanges
    // class has Install/Upgrade/Reinstall/Downgrade/Remove/Purge as distinct
    // categories (confirmed via `strings` on the real shared library on this
    // host) -- history.log is not limited to the three headers this parser
    // originally recognized. A block containing only one of the other three
    // real headers used to produce an entry with a correct date/commandline
    // but a silently EMPTY summary.
    #[test]
    fn recognizes_a_purge_only_block() {
        let content = "Start-Date: 2026-08-04  10:00:00\nCommandline: apt-get purge -y flatpak\nPurge: flatpak:amd64 (1.16.6-1)\nEnd-Date: 2026-08-04  10:00:01\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].summary.is_empty(), "a real Purge: block must not produce an empty summary");
        assert!(entries[0].summary.contains("flatpak"));
    }

    #[test]
    fn recognizes_a_reinstall_only_block() {
        let content = "Start-Date: 2026-08-04  10:05:00\nCommandline: apt-get install --reinstall -y curl\nReinstall: curl:amd64 (7.88.1-10)\nEnd-Date: 2026-08-04  10:05:01\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].summary.is_empty());
        assert!(entries[0].summary.contains("curl"));
    }

    #[test]
    fn recognizes_a_downgrade_only_block() {
        let content = "Start-Date: 2026-08-04  10:10:00\nCommandline: apt-get install -y curl=7.88.1-9\nDowngrade: curl:amd64 (7.88.1-10, 7.88.1-9)\nEnd-Date: 2026-08-04  10:10:01\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].summary.is_empty());
        assert!(entries[0].summary.contains("curl"));
    }
}
