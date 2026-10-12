//! A log of every administrator (pkexec) action NiTruX ran: what, when,
//! and whether it worked. One JSON object per line in
//! `$XDG_DATA_HOME/nitrux/admin-history.jsonl`, the newest 500 kept.
//! Content sent on stdin (hosts file, DNS) is never logged, only the action.

use serde::{Deserialize, Serialize};
use std::io::Write;

const KEEP: usize = 500;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AdminAction {
    pub timestamp_ms: u64,
    /// The helper's action and arguments, e.g. "install-package pacman htop".
    pub action: String,
    pub ok: bool,
    /// First line of the error, empty on success.
    pub detail: String,
}

fn describe(args: &[&str]) -> String {
    // args[0] is the helper path (/usr/bin/nitrux-pkexec-...): the
    // subcommand that follows already says what it is.
    let text = args.iter().skip(1).copied().collect::<Vec<_>>().join(" ");
    // A script run as root can be long: its start says what it was.
    match text.char_indices().nth(200) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text,
    }
}

/// Best effort: a log that cannot be written must never fail the action.
pub fn record(args: &[&str], result: Result<(), &str>) {
    let entry = AdminAction {
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        action: describe(args),
        ok: result.is_ok(),
        detail: result.err().and_then(|e| e.lines().next()).unwrap_or("").to_string(),
    };
    let Ok(dir) = crate::perf_history::data_dir() else { return };
    let path = dir.join("admin-history.jsonl");
    let Ok(line) = serde_json::to_string(&entry) else { return };
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{line}");
    }
    trim(&path);
}

fn trim(path: &std::path::Path) {
    let Ok(contents) = std::fs::read_to_string(path) else { return };
    let lines: Vec<&str> = contents.lines().collect();
    if lines.len() > KEEP * 2 {
        let _ = std::fs::write(path, lines[lines.len() - KEEP..].join("\n") + "\n");
    }
}

fn parse(contents: &str) -> Vec<AdminAction> {
    let mut entries: Vec<AdminAction> = contents.lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    entries.reverse(); // newest first
    entries.truncate(KEEP);
    entries
}

#[tauri::command]
pub fn get_admin_history() -> Result<Vec<AdminAction>, String> {
    let path = crate::perf_history::data_dir()?.join("admin-history.jsonl");
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(parse(&contents)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("lecture de l'historique impossible : {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_the_action_without_the_helper_path() {
        assert_eq!(describe(&["/usr/bin/nitrux-pkexec-install-package", "install-package", "pacman", "htop"]), "install-package pacman htop");
    }

    #[test]
    fn records_and_reads_back_newest_first() {
        let _guard = crate::TEST_ENV_LOCK.lock().unwrap();
        let scratch = std::env::temp_dir().join(format!("nitrux-admin-{}", std::process::id()));
        let previous = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &scratch);
        record(&["/usr/bin/h", "install-package", "apt", "vlc"], Ok(()));
        record(&["/usr/bin/h", "set-dns"], Err("pkexec a échoué (code 126) : authentification annulée\nsuite"));
        let history = get_admin_history().unwrap();
        match previous {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        let _ = std::fs::remove_dir_all(&scratch);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].action, "set-dns");
        assert!(!history[0].ok);
        assert_eq!(history[0].detail, "pkexec a échoué (code 126) : authentification annulée");
        assert_eq!(history[1].action, "install-package apt vlc");
        assert!(history[1].ok);
    }

    #[test]
    fn skips_a_torn_line() {
        assert_eq!(parse("{\"timestamp_ms\":1,\"action\":\"a\",\"ok\":true,\"detail\":\"\"}\n{\"timest").len(), 1);
    }
}
