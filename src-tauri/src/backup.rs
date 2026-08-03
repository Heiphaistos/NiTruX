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
    subprocess::run_with_timeout(
        "tar",
        &["-czf", &dest_path, &exclude_arg, "-C", &source_dir, "."],
        Duration::from_secs(300),
    )?;

    Ok(dest_path)
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
}
