use crate::subprocess;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

/// Rejects any source path that is not absolute, or that attempts to
/// escape via `..` -- this runs unprivileged (no pkexec), but a
/// non-absolute or traversal-laden path is still worth rejecting outright
/// as an obvious misuse rather than letting `tar` interpret it.
pub fn validate_source_dir(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin source vide".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("le chemin source doit être absolu : {path}"));
    }
    if path.contains("..") {
        return Err(format!("le chemin source ne doit pas contenir '..' : {path}"));
    }
    Ok(())
}

pub fn backup_filename(now_epoch_secs: u64) -> String {
    format!("nitrux-backup-{now_epoch_secs}.tar.gz")
}

/// GNU tar's real exit-code convention: 0 = no issues, 1 = "some files
/// differ" -- e.g. a file changed size while being read, which is a real,
/// common occurrence when archiving a live, in-use directory (browser
/// profile databases, logs, caches, swap files...) and NOT specific to the
/// self-referential archive-writing-itself case `--exclude` already
/// guards against below. The archive is still written successfully in
/// this case (confirmed live: tar prints "File shrank by N bytes; padding
/// with zeros" to stderr and still produces a valid, listable .tar.gz) --
/// only exit codes >= 2 are fatal (e.g. invalid path, disk full,
/// permission denied). Previously `run_with_timeout` treated exit 1 as a
/// hard `Err`, so backing up a real $HOME (this feature's primary use
/// case) would very plausibly report "backup failed" to the user even
/// though a good archive was sitting right there on disk.
///
/// Fatal errors (code >= 2) include `stderr` in the message: reproduced
/// live (an unreadable file inside the source directory, chmod 000) that
/// GNU tar prints its actionable reason ("Cannot open: Permission denied")
/// to stderr with an EMPTY stdout, so a message built from stdout alone (or
/// from the code alone, as before this fix) would silently drop the one
/// piece of information the user actually needs to fix the problem.
fn interpret_tar_result(dest_path: String, stderr: &str, code: i32) -> Result<String, String> {
    match code {
        0 | 1 => Ok(dest_path),
        _ if stderr.trim().is_empty() => Err(format!("tar a rencontré une erreur (code {code})")),
        _ => Err(format!("tar a rencontré une erreur (code {code}) : {}", stderr.trim())),
    }
}

#[tauri::command]
pub fn create_backup(source_dir: String) -> Result<String, String> {
    validate_source_dir(&source_dir)?;
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    let now_epoch_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let filename = backup_filename(now_epoch_secs);
    let dest_path = format!("{home}/{filename}");

    // The archive is always written into $HOME (see the description in
    // BackupPage.vue), but the source directory can legitimately BE $HOME
    // (backing up the whole home folder is a natural thing to type). Without
    // this exclude, tar would try to read the very file it's writing to as
    // it walks the tree -- reproduced live: GNU tar prints "file changed as
    // we read it" to stderr (silently discarded by run_with_timeout on
    // success, so the user never sees it) for that entry. --exclude, not a
    // post-hoc filter, is the fix: it stops tar from ever opening the file
    // mid-write, which is more robust than relying on however the archive
    // happened to come out this time.
    let exclude_arg = format!("--exclude={filename}");
    let (_, stderr, code) = subprocess::run_capturing_exit_code(
        "tar",
        &["-czf", &dest_path, &exclude_arg, "-C", &source_dir, "."],
        Duration::from_secs(300),
    )?;

    let dest_path = interpret_tar_result(dest_path, &stderr, code)?;
    restrict_backup_to_owner(&dest_path)?;
    Ok(dest_path)
}

/// `tar` creates its output file with whatever the process umask leaves
/// (0022 on this dev host, confirmed live -- the archive comes out
/// world-readable, `-rw-r--r--`). A backup of a user-chosen directory can
/// legitimately contain highly sensitive data (SSH private keys, browser
/// profiles with saved credentials/cookies, documents, mail) -- and
/// $HOME, where this archive always lands, is traversable by other local
/// users on a default Debian install (`adduser`'s DEFAULT_HOME_PERMS is
/// 0755). Restricting to owner-only after the fact closes that gap;
/// failing to do so is surfaced as an error rather than silently leaving
/// a sensitive archive world-readable, since the whole point of this step
/// is the confidentiality guarantee itself.
fn restrict_backup_to_owner(path: &str) -> Result<(), String> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("archive créée mais impossible de restreindre ses permissions : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_well_formed_absolute_source_path() {
        assert!(validate_source_dir("/home/dev/documents").is_ok());
    }

    #[test]
    fn rejects_empty_source_path() {
        assert!(validate_source_dir("").is_err());
    }

    #[test]
    fn rejects_relative_source_path() {
        assert!(validate_source_dir("documents").is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_source_dir("/home/dev/../../etc").is_err());
    }

    #[test]
    fn backup_filename_includes_the_epoch_timestamp() {
        assert_eq!(backup_filename(1735689600), "nitrux-backup-1735689600.tar.gz");
    }

    #[test]
    fn exit_code_1_still_reports_success_with_the_archive_path() {
        // Reproduced live: tar exits 1 (not 0) when a file changes size
        // while being archived ("File shrank by N bytes; padding with
        // zeros" on stderr) -- the archive is still written successfully,
        // this must not be reported as a failed backup.
        let result = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), "", 1)
            .expect("exit 1 (files changed during archiving) should be Ok, not an error");
        assert_eq!(result, "/home/dev/nitrux-backup-1.tar.gz");
    }

    #[test]
    fn exit_code_0_reports_success() {
        let result = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), "", 0)
            .expect("exit 0 should be Ok");
        assert_eq!(result, "/home/dev/nitrux-backup-1.tar.gz");
    }

    #[test]
    fn exit_code_2_is_a_real_fatal_error() {
        let err = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), "", 2)
            .expect_err("exit 2 should be a real error");
        assert!(err.contains('2'), "error should mention the exit code: {err}");
    }

    #[test]
    fn fatal_error_includes_the_real_stderr_reason_when_present() {
        // Reproduced live: an unreadable file (chmod 000) inside the source
        // directory makes tar exit 2 with its actionable explanation ONLY
        // on stderr, stdout empty.
        let err = interpret_tar_result(
            "/home/dev/nitrux-backup-1.tar.gz".to_string(),
            "tar: ./f.txt: Cannot open: Permission denied\ntar: Exiting with failure status due to previous errors\n",
            2,
        )
        .expect_err("exit 2 should be a real error");
        assert!(
            err.contains("Permission denied"),
            "the real reason from stderr must not be silently dropped: {err}"
        );
    }

    #[test]
    fn restrict_backup_to_owner_removes_group_and_world_access() {
        // Reproduced live: `tar -czf` leaves its output at the process's
        // umask (0022 on this host), i.e. world-readable -- exactly what
        // this function must close for a file that can contain SSH keys,
        // browser credentials, or other sensitive personal data.
        let path = std::env::temp_dir().join(format!("nitrux-backup-test-perms-{}.tar.gz", std::process::id()));
        std::fs::write(&path, b"fake archive content").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        restrict_backup_to_owner(path.to_str().unwrap()).expect("should succeed");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        std::fs::remove_file(&path).ok();

        assert_eq!(mode, 0o600, "backup archive must be readable/writable only by its owner");
    }
}
