use crate::subprocess;
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
fn interpret_tar_result(dest_path: String, code: i32) -> Result<String, String> {
    match code {
        0 | 1 => Ok(dest_path),
        _ => Err(format!("tar a rencontré une erreur (code {code})")),
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
    let (_, code) = subprocess::run_capturing_exit_code(
        "tar",
        &["-czf", &dest_path, &exclude_arg, "-C", &source_dir, "."],
        Duration::from_secs(300),
    )?;

    interpret_tar_result(dest_path, code)
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
        let result = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), 1)
            .expect("exit 1 (files changed during archiving) should be Ok, not an error");
        assert_eq!(result, "/home/dev/nitrux-backup-1.tar.gz");
    }

    #[test]
    fn exit_code_0_reports_success() {
        let result = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), 0)
            .expect("exit 0 should be Ok");
        assert_eq!(result, "/home/dev/nitrux-backup-1.tar.gz");
    }

    #[test]
    fn exit_code_2_is_a_real_fatal_error() {
        let err = interpret_tar_result("/home/dev/nitrux-backup-1.tar.gz".to_string(), 2)
            .expect_err("exit 2 should be a real error");
        assert!(err.contains('2'), "error should mention the exit code: {err}");
    }
}
