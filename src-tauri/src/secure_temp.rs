//! Shared helper for writing a file under the shared temp directory
//! without the CWE-377 (predictable name)/CWE-367 (TOCTOU) shape that a
//! plain `std::fs::write`/`File::create` has there: both silently follow
//! whatever already exists at the target path -- including a symlink an
//! attacker (or, for `pkexec_bootstrap`'s caller specifically, same-user
//! malware timing the write-then-execute-as-root window) planted in
//! advance -- and leave the file world-readable by default. Found and
//! fixed first in `pkexec_bootstrap.rs` (2026-08-06), where the impact is
//! most severe (the file is executed as root moments later); reused here
//! for every other temp file this codebase writes to a predictable,
//! PID-based path, since the same code smell has the same fix regardless
//! of whether the specific caller happens to run the result with elevated
//! privileges afterward.

use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// Opens `path` for writing, created exclusively (fails instead of
/// following an existing file or symlink at that path) with permissions
/// restricted to the owner (`0o600`). A best-effort `remove_file` first
/// clears both a stale leftover from a prior run that never reached its
/// own cleanup and any pre-positioned attacker symlink -- either way the
/// exclusive create right after only proceeds if the path is genuinely
/// clear at that instant. Returns the open `File` for callers that need
/// to do more than a single `write_all` with it (e.g. timing the write,
/// or calling `sync_all` themselves) -- see `write_exclusively_owner_only`
/// below for the common "just write this content" case.
pub fn create_exclusively_owner_only(path: &Path) -> Result<std::fs::File, String> {
    let _ = std::fs::remove_file(path);
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| format!("impossible de créer le fichier temporaire : {e}"))
}

/// Writes `content` to `path` in one shot -- see `create_exclusively_owner_only`
/// for what makes this safe against a predictable-path attack.
pub fn write_exclusively_owner_only(path: &Path, content: &[u8]) -> Result<(), String> {
    let mut file = create_exclusively_owner_only(path)?;
    file.write_all(content).map_err(|e| format!("impossible d'écrire le fichier temporaire : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_the_given_content_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!("nitrux-secure-temp-test-{}.txt", std::process::id()));
        write_exclusively_owner_only(&path, b"hello").expect("should write");
        let content = std::fs::read(&path).expect("should read back");
        let mode = std::fs::metadata(&path).expect("should stat").permissions().mode() & 0o777;
        std::fs::remove_file(&path).ok();
        assert_eq!(content, b"hello");
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn clears_a_preexisting_symlink_instead_of_following_it() {
        let canary_dir = std::env::temp_dir().join(format!("nitrux-secure-temp-canary-{}", std::process::id()));
        std::fs::create_dir_all(&canary_dir).unwrap();
        let canary = canary_dir.join("canary.txt");
        std::fs::write(&canary, "untouched").unwrap();

        let target = std::env::temp_dir().join(format!("nitrux-secure-temp-symlink-{}.txt", std::process::id()));
        std::os::unix::fs::symlink(&canary, &target).expect("should create the pre-positioned symlink");

        let result = write_exclusively_owner_only(&target, b"new content");

        let canary_content = std::fs::read_to_string(&canary).unwrap();
        let target_is_symlink = std::fs::symlink_metadata(&target).map(|m| m.file_type().is_symlink()).unwrap_or(false);
        std::fs::remove_file(&target).ok();
        std::fs::remove_dir_all(&canary_dir).ok();

        result.expect("should succeed despite the pre-positioned symlink");
        assert_eq!(canary_content, "untouched");
        assert!(!target_is_symlink);
    }

    #[test]
    fn a_second_write_to_the_same_path_still_succeeds() {
        // Mirrors normal operation: a caller that cleans up its own file
        // after use (as both benchmark_disk and
        // pkexec_bootstrap::write_bootstrap_script_to_temp do) must be
        // able to call this again later without erroring on a leftover.
        let path = std::env::temp_dir().join(format!("nitrux-secure-temp-reuse-{}.txt", std::process::id()));
        write_exclusively_owner_only(&path, b"first").expect("first write should succeed");
        write_exclusively_owner_only(&path, b"second").expect("second write should succeed, not error on the leftover");
        let content = std::fs::read(&path).expect("should read back");
        std::fs::remove_file(&path).ok();
        assert_eq!(content, b"second");
    }
}
