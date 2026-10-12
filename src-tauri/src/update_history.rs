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

/// `/var/log/pacman.log`: each "[PACMAN] Running '...'" line opens a
/// transaction, the "[ALPM] installed|upgraded|removed|downgraded|
/// reinstalled <pkg> (<version>)" lines after it are its content. At most
/// a dozen names per action are listed: an Arch install is one transaction
/// of nearly a thousand packages.
pub fn parse_pacman_log(content: &str) -> Vec<UpdateHistoryEntry> {
    const ACTIONS: [(&str, &str); 5] = [
        ("installed", "Install"),
        ("upgraded", "Upgrade"),
        ("downgraded", "Downgrade"),
        ("reinstalled", "Reinstall"),
        ("removed", "Remove"),
    ];
    let mut entries = Vec::new();
    let mut current: Option<(String, String, Vec<Vec<String>>)> = None;
    let flush = |current: Option<(String, String, Vec<Vec<String>>)>, entries: &mut Vec<UpdateHistoryEntry>| {
        if let Some((start_date, commandline, groups)) = current {
            let summary = ACTIONS
                .iter()
                .zip(&groups)
                .filter(|(_, names)| !names.is_empty())
                .map(|((_, label), names)| {
                    let shown = names.iter().take(12).cloned().collect::<Vec<_>>().join(", ");
                    let more = if names.len() > 12 { format!(" … (+{})", names.len() - 12) } else { String::new() };
                    format!("{label}: {shown}{more}")
                })
                .collect::<Vec<_>>()
                .join(" | ");
            if !summary.is_empty() {
                entries.push(UpdateHistoryEntry { start_date, commandline, summary });
            }
        }
    };
    for line in content.lines() {
        let Some((stamp, rest)) = line.strip_prefix('[').and_then(|l| l.split_once("] ")) else { continue };
        if let Some(cmd) = rest.strip_prefix("[PACMAN] Running '") {
            flush(current.take(), &mut entries);
            current = Some((stamp.replace('T', " "), cmd.trim_end_matches('\'').to_string(), vec![Vec::new(); ACTIONS.len()]));
        } else if let (Some(alpm), Some((_, _, groups))) = (rest.strip_prefix("[ALPM] "), current.as_mut()) {
            for (i, (verb, _)) in ACTIONS.iter().enumerate() {
                if let Some(pkg) = alpm.strip_prefix(verb).and_then(|p| p.strip_prefix(' ')) {
                    groups[i].push(pkg.to_string());
                }
            }
        }
    }
    flush(current, &mut entries);
    entries
}

#[tauri::command]
pub fn get_update_history() -> Result<Vec<UpdateHistoryEntry>, String> {
    if std::path::Path::new("/var/log/pacman.log").exists() && !std::path::Path::new("/var/log/apt/history.log").exists() {
        let content = std::fs::read_to_string("/var/log/pacman.log")
            .map_err(|e| format!("impossible de lire l'historique : {e}"))?;
        let mut entries = parse_pacman_log(&content);
        entries.reverse(); // most recent first
        return Ok(entries);
    }
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
    fn groups_pacman_log_lines_into_transactions() {
        // Lines captured from /var/log/pacman.log on Omarchy.
        let log = "[2026-10-12T08:41:48+0200] [PACMAN] Running 'pacman -S --noconfirm --needed bitwarden bitwarden-cli'\n\
[2026-10-12T08:41:50+0200] [ALPM] transaction started\n\
[2026-10-12T08:41:52+0200] [ALPM] installed bitwarden (2026.9.0-1)\n\
[2026-10-12T08:41:52+0200] [ALPM] upgraded electron (39.1.0-1 -> 39.2.0-1)\n\
[2026-10-12T08:41:53+0200] [ALPM] running 'gtk-update-icon-cache.hook'...\n\
[2026-10-12T09:00:00+0200] [PACMAN] Running 'pacman -Sy'\n\
[2026-10-12T09:00:01+0200] [PACMAN] synchronizing package lists\n";
        let entries = parse_pacman_log(log);
        assert_eq!(entries.len(), 1, "a sync with nothing installed is not a history entry");
        assert_eq!(entries[0].start_date, "2026-10-12 08:41:48+0200");
        assert_eq!(entries[0].commandline, "pacman -S --noconfirm --needed bitwarden bitwarden-cli");
        assert_eq!(entries[0].summary, "Install: bitwarden (2026.9.0-1) | Upgrade: electron (39.1.0-1 -> 39.2.0-1)");
    }

    #[test]
    fn reads_this_hosts_real_pacman_log_when_there_is_one() {
        let Ok(content) = std::fs::read_to_string("/var/log/pacman.log") else { return };
        let entries = parse_pacman_log(&content);
        assert!(!entries.is_empty(), "a real pacman.log should yield at least one transaction");
        assert!(entries.iter().all(|e| e.commandline.starts_with("pacman")));
    }

    #[test]
    fn caps_huge_pacman_transactions() {
        let mut log = String::from("[2026-10-12T07:04:41+0000] [PACMAN] Running 'pacman -S base'\n");
        for i in 0..20 {
            log.push_str(&format!("[2026-10-12T07:04:44+0000] [ALPM] installed pkg{i} (1-1)\n"));
        }
        assert!(parse_pacman_log(&log)[0].summary.ends_with("… (+8)"));
    }

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
    fn does_not_truncate_a_very_long_single_line_install_block() {
        // Investigated a suspected bug: does apt wrap long Install:/
        // Upgrade: lines across multiple physical lines (continuation
        // lines starting with whitespace, no repeated prefix), which this
        // parser would silently drop since it only matches lines starting
        // with one of SUMMARY_PREFIXES? Verified live against a real
        // history.log on the VM: a genuine `apt full-upgrade` Install:
        // line there is 96287 characters long, entirely on ONE physical
        // line -- apt never wraps history.log entries, no matter how many
        // packages are listed. The hypothesis was wrong; this pins the
        // real, verified behavior as a regression guard instead.
        let mut install_line = String::from("Install: ");
        for i in 0..500 {
            install_line.push_str(&format!("pkg{i}:amd64 (1.0-{i}, automatic), "));
        }
        let content = format!("Start-Date: 2026-08-04  12:00:00\nCommandline: apt-get full-upgrade -y\n{install_line}\nEnd-Date: 2026-08-04  12:00:30\n");
        let entries = parse_apt_history(&content);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].summary.contains("pkg0:amd64"), "first package must be present");
        assert!(entries[0].summary.contains("pkg499:amd64"), "last package must be present, not truncated");
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
