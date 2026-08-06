//! Tauri commands for the privileged troubleshoot / snapshot / quarantine
//! write operations, shelling out to `pkexec` against the dedicated helper
//! binaries installed by `packaging/nitrux-pkexec-helper` (see that script's
//! header comment for the full contract). Each command validates its input
//! here too, even though the helper script re-validates independently as
//! root and never trusts the caller — this Rust-side check exists purely to
//! give the user a clear, immediate error instead of an opaque failure from
//! `pkexec`/the helper for obviously malformed input.

use crate::subprocess;
use std::time::Duration;

const PKEXEC_TROUBLESHOOT: &str = "/usr/bin/nitrux-pkexec-troubleshoot";
const PKEXEC_CREATE_SNAPSHOT: &str = "/usr/bin/nitrux-pkexec-create-snapshot";
const PKEXEC_QUARANTINE_FILE: &str = "/usr/bin/nitrux-pkexec-quarantine-file";

pub fn validate_troubleshoot_action(action: &str) -> Result<(), String> {
    match action {
        "clean-cache" | "fix-broken" | "restart-network" | "vacuum-logs" => Ok(()),
        other => Err(format!("action de dépannage inconnue : {other}")),
    }
}

/// Mirrors the helper script's own `validate_quarantine_path`, checked
/// here too so obviously-malformed input gets a clear error from the app
/// itself rather than an opaque failure from `pkexec`/the helper.
///
/// Rejects the root directory itself (`/`, `//`, ...) on top of what the
/// helper script's own copy of this check does: read there (never run --
/// modifying a pkexec-backed script requires live VM re-verification per
/// this project's standing rule, unavailable this cycle), `/` passes ALL
/// of its checks (absolute, no `..`, no metacharacters) exactly like it
/// does here, then `basename "/"` prints `/`, so `quarantine-file /` would
/// build a destination of `.../<timestamp>-/` and attempt `mv / <dest>` as
/// root -- a "quarantine one file" feature has no legitimate reason to
/// ever target the filesystem root. Confirmed by reading, not exploited.
pub fn validate_quarantine_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin de quarantaine vide".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("le chemin doit être absolu : {path}"));
    }
    if path.trim_end_matches('/').is_empty() {
        return Err("impossible de mettre en quarantaine la racine du système de fichiers".to_string());
    }
    if path.contains("..") {
        return Err(format!("le chemin ne doit pas contenir '..' : {path}"));
    }
    const DISALLOWED: [char; 10] = ['*', '?', '$', '`', ';', '|', '&', '<', '>', '\n'];
    if path.chars().any(|c| DISALLOWED.contains(&c)) {
        return Err(format!("le chemin contient des caractères non autorisés : {path}"));
    }
    Ok(())
}

#[tauri::command]
pub fn run_troubleshoot_action(action: String) -> Result<String, String> {
    validate_troubleshoot_action(&action)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_TROUBLESHOOT, "troubleshoot", &action],
        Duration::from_secs(120),
    )
}

#[tauri::command]
pub fn create_snapshot() -> Result<String, String> {
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_CREATE_SNAPSHOT, "create-snapshot"],
        Duration::from_secs(600),
    )
}

#[tauri::command]
pub fn quarantine_file(path: String) -> Result<String, String> {
    validate_quarantine_path(&path)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_QUARANTINE_FILE, "quarantine-file", &path],
        Duration::from_secs(30),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_troubleshoot_actions() {
        assert!(validate_troubleshoot_action("clean-cache").is_ok());
        assert!(validate_troubleshoot_action("fix-broken").is_ok());
        assert!(validate_troubleshoot_action("restart-network").is_ok());
        assert!(validate_troubleshoot_action("vacuum-logs").is_ok());
    }

    #[test]
    fn rejects_unknown_troubleshoot_action() {
        assert!(validate_troubleshoot_action("rm-rf-root").is_err());
        assert!(validate_troubleshoot_action("").is_err());
        assert!(validate_troubleshoot_action("clean-cache; rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_absolute_quarantine_path() {
        assert!(validate_quarantine_path("/home/dev/eicar.txt").is_ok());
        assert!(validate_quarantine_path("/tmp/suspicious.bin").is_ok());
    }

    #[test]
    fn rejects_relative_or_empty_quarantine_path() {
        assert!(validate_quarantine_path("").is_err());
        assert!(validate_quarantine_path("relative/path.txt").is_err());
        assert!(validate_quarantine_path("../etc/passwd").is_err());
    }

    #[test]
    fn rejects_the_filesystem_root() {
        // "/" passes every other check here (absolute, no "..", no
        // metacharacters) and, read (not run) in the pkexec helper script,
        // `basename "/"` is "/" too -- would build mv's destination as
        // ".../<timestamp>-/" and attempt to mv the whole root as root.
        let err = validate_quarantine_path("/").expect_err("root must be rejected");
        assert!(err.contains("racine"), "error should explain why: {err}");
        assert!(validate_quarantine_path("//").is_err(), "repeated slashes must also collapse to root");
        assert!(validate_quarantine_path("///").is_err());
    }

    #[test]
    fn rejects_quarantine_path_with_traversal_or_metacharacters() {
        assert!(validate_quarantine_path("/home/dev/../../etc/shadow").is_err());
        assert!(validate_quarantine_path("/tmp/evil;rm -rf /").is_err());
        assert!(validate_quarantine_path("/tmp/$(whoami)").is_err());
        assert!(validate_quarantine_path("/tmp/`whoami`").is_err());
    }
}
