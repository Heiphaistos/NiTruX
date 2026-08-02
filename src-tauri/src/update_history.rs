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
/// containing "Commandline:" plus one or more of
/// "Install:"/"Upgrade:"/"Remove:", ending with "End-Date:". Malformed or
/// incomplete blocks (missing Start-Date or Commandline) are skipped
/// rather than producing a partial/garbage entry.
pub fn parse_apt_history(content: &str) -> Vec<UpdateHistoryEntry> {
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
            } else if line.starts_with("Install: ") || line.starts_with("Upgrade: ") || line.starts_with("Remove: ") {
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
}
