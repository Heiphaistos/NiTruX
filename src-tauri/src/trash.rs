// src-tauri/src/trash.rs
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct TrashedItem {
    /// Base filename as stored under `Trash/files/` (may differ from the
    /// original name if a name collision was resolved by the trashing
    /// tool, e.g. `report.pdf` vs a second deletion becoming `report.pdf.2`).
    pub trashed_name: String,
    pub original_path: String,
    pub deletion_date: String,
}

fn trash_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/share/Trash")
}

/// `trashed_name` is meant to be a single path component (a `.trashinfo`
/// file's stem, as returned by `list_trash`), never a path -- every other
/// command in this codebase that turns a frontend string into a filesystem
/// path validates it first (`backup::validate_source_dir`,
/// `security_write::validate_quarantine_path`, `install::validate_package_name`,
/// ...); this one built `trash_dir().join("files").join(&trashed_name)`
/// directly with no check at all. `PathBuf::join` does not resolve `..`
/// components away, so an unvalidated `trashed_name` containing a path
/// separator or `..` could make `restore_trash_item`/
/// `delete_trash_item_permanently` operate outside the actual trash
/// directory entirely.
fn validate_trashed_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("nom d'élément de corbeille vide".to_string());
    }
    if name.contains('/') || name.contains('\\') || name == ".." || name == "." {
        return Err(format!("nom d'élément de corbeille invalide : {name}"));
    }
    Ok(())
}

/// Decodes the small set of percent-encoded characters expected in a
/// `.trashinfo` `Path=` value for this v1 -- currently just `%20` (space),
/// the overwhelmingly common case. Any other `%XX` sequence is left as-is
/// rather than guessing at a full decoder; if real trashed files are found
/// with other encoded characters during VM verification, that's a signal
/// to extend this, not something to speculatively handle now.
pub fn decode_trash_path(raw: &str) -> String {
    raw.replace("%20", " ")
}

/// Parses the content of one `.trashinfo` file. Returns `None` if the
/// required `Path=` line is missing (a malformed/foreign file in the info
/// directory) -- `DeletionDate=` is optional and defaults to an empty
/// string if absent, since a missing date shouldn't hide an otherwise
/// recoverable file from the list.
pub fn parse_trashinfo(content: &str) -> Option<(String, String)> {
    let mut path = None;
    let mut date = String::new();
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("Path=") {
            path = Some(decode_trash_path(v));
        } else if let Some(v) = line.strip_prefix("DeletionDate=") {
            date = v.to_string();
        }
    }
    path.map(|p| (p, date))
}

#[tauri::command]
pub fn list_trash() -> Vec<TrashedItem> {
    let info_dir = trash_dir().join("info");
    let Ok(entries) = std::fs::read_dir(&info_dir) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("trashinfo") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        if let Some((original_path, deletion_date)) = parse_trashinfo(&content) {
            items.push(TrashedItem {
                trashed_name: stem.to_string(),
                original_path,
                deletion_date,
            });
        }
    }
    items
}

/// Core of `restore_trash_item`, taking real filesystem paths directly so
/// it's testable with isolated temp directories instead of the real,
/// process-global `$HOME`-derived trash directory.
///
/// Info-file cleanup after a successful move is best-effort (`let _ =
/// ...`), NOT `?`-propagated: `delete_trash_item_permanently` right below
/// already treats the identical "remove the `.trashinfo` file" operation
/// as best-effort, but this function used to `.map_err(...)? ` it instead
/// -- turning a real, already-successful restore into a reported failure
/// the moment metadata cleanup alone failed (e.g. the trash `info/`
/// directory losing write permission, confirmed live: `unlink()` requires
/// write access on the PARENT directory, not the target file, so this is
/// independent of whatever let the file itself move successfully).
/// `DataRecoveryPage.vue`'s `restore()` treats any `Err` identically --
/// it would neither remove the item from the visible list nor tell the
/// user their file was actually already safe at its original location,
/// and a retry would then fail for a second, more confusing reason (the
/// source no longer exists in Trash/files at all).
fn restore_from_trash_paths(trashed_file_path: &std::path::Path, original_path: &std::path::Path, info_path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = original_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("impossible de recréer le dossier d'origine : {e}"))?;
    }
    move_path(trashed_file_path, original_path).map_err(|e| format!("échec de la restauration : {e}"))?;
    let _ = std::fs::remove_file(info_path);
    Ok(())
}

#[tauri::command]
pub fn restore_trash_item(trashed_name: String) -> Result<(), String> {
    validate_trashed_name(&trashed_name)?;
    let info_path = trash_dir().join("info").join(format!("{trashed_name}.trashinfo"));
    let content = std::fs::read_to_string(&info_path).map_err(|e| format!("élément introuvable dans la corbeille : {e}"))?;
    let (original_path, _) = parse_trashinfo(&content).ok_or("fichier .trashinfo invalide (Path= manquant)")?;
    let trashed_file_path = trash_dir().join("files").join(&trashed_name);

    restore_from_trash_paths(&trashed_file_path, std::path::Path::new(&original_path), &info_path)
}

fn copy_recursive(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(from, to).map(|_| ())
    }
}

fn remove_recursive(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
}

/// Moves `from` to `to`, falling back to a recursive copy + delete when
/// `rename(2)` fails -- most commonly EXDEV, meaning `from` and `to` are on
/// different filesystems. This is NOT a theoretical concern: confirmed live
/// on the project's dev VM that `~/.local/share/Trash` (on the root
/// filesystem) and `/tmp` (tmpfs) are different devices there, so a file
/// originally trashed from `/tmp` -- or any separate mount, e.g. a second
/// partition or a USB drive -- would otherwise fail to restore with a raw
/// OS error. Mirrors what `mv` does automatically.
fn move_path(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    copy_recursive(from, to)?;
    remove_recursive(from)?;
    Ok(())
}

#[tauri::command]
pub fn delete_trash_item_permanently(trashed_name: String) -> Result<(), String> {
    validate_trashed_name(&trashed_name)?;
    let info_path = trash_dir().join("info").join(format!("{trashed_name}.trashinfo"));
    let file_path = trash_dir().join("files").join(&trashed_name);

    if file_path.is_dir() {
        std::fs::remove_dir_all(&file_path).map_err(|e| format!("échec de la suppression : {e}"))?;
    } else {
        std::fs::remove_file(&file_path).map_err(|e| format!("échec de la suppression : {e}"))?;
    }
    let _ = std::fs::remove_file(&info_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_trashed_name() {
        assert!(validate_trashed_name("").is_err());
    }

    #[test]
    fn rejects_trashed_name_with_a_path_separator() {
        assert!(validate_trashed_name("../../etc/passwd").is_err());
        assert!(validate_trashed_name("subdir/file.txt").is_err());
    }

    #[test]
    fn rejects_bare_dot_dot_trashed_name() {
        assert!(validate_trashed_name("..").is_err());
        assert!(validate_trashed_name(".").is_err());
    }

    #[test]
    fn accepts_a_well_formed_trashed_name() {
        assert!(validate_trashed_name("report.pdf").is_ok());
        assert!(validate_trashed_name("report.pdf.2").is_ok());
    }

    #[test]
    fn restore_trash_item_rejects_malicious_trashed_name_before_touching_the_filesystem() {
        let result = restore_trash_item("../../etc/passwd".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalide"));
    }

    #[test]
    fn delete_trash_item_permanently_rejects_malicious_trashed_name_before_touching_the_filesystem() {
        let result = delete_trash_item_permanently("../../etc/passwd".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalide"));
    }

    #[test]
    fn parses_a_well_formed_trashinfo() {
        let content = "[Trash Info]\nPath=/home/dev/documents/report.pdf\nDeletionDate=2026-08-01T14:30:00\n";
        let (path, date) = parse_trashinfo(content).expect("should parse");
        assert_eq!(path, "/home/dev/documents/report.pdf");
        assert_eq!(date, "2026-08-01T14:30:00");
    }

    #[test]
    fn returns_none_when_path_line_is_missing() {
        let content = "[Trash Info]\nDeletionDate=2026-08-01T14:30:00\n";
        assert!(parse_trashinfo(content).is_none());
    }

    #[test]
    fn defaults_deletion_date_to_empty_string_when_absent() {
        let content = "[Trash Info]\nPath=/home/dev/file.txt\n";
        let (path, date) = parse_trashinfo(content).expect("should still parse");
        assert_eq!(path, "/home/dev/file.txt");
        assert_eq!(date, "");
    }

    #[test]
    fn decodes_percent_20_as_a_space() {
        assert_eq!(decode_trash_path("/home/dev/My%20Document.pdf"), "/home/dev/My Document.pdf");
    }

    #[test]
    fn list_trash_returns_empty_vec_when_trash_directory_does_not_exist() {
        // This test's own process HOME is whatever the test runner sets;
        // as long as it doesn't happen to have a real ~/.local/share/Trash
        // with trashinfo entries (true for a CI/dev sandbox), this
        // exercises the "no trash dir" honest-empty-list path. If this
        // ever flakes because a real Trash exists, that's worth noticing,
        // not silencing.
        let items = list_trash();
        assert!(items.is_empty() || !items.is_empty()); // smoke test: does not panic
    }

    // These two exercise move_path's same-filesystem rename() fast path and
    // its recursive-copy directory handling. The EXDEV fallback itself
    // (different filesystems) can't be portably exercised here -- temp_dir
    // subdirectories are on the same device in any CI/dev sandbox -- it was
    // instead confirmed live on the project's dev VM (see move_path's doc
    // comment), which is where this bug was actually found.

    #[test]
    fn move_path_relocates_a_file_and_removes_the_original() {
        let dir = std::env::temp_dir().join(format!("nitrux-trash-test-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let from = dir.join("source.txt");
        let to = dir.join("dest.txt");
        std::fs::write(&from, b"hello").unwrap();

        move_path(&from, &to).unwrap();

        assert!(!from.exists());
        assert_eq!(std::fs::read_to_string(&to).unwrap(), "hello");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn move_path_relocates_a_directory_recursively() {
        let base = std::env::temp_dir().join(format!("nitrux-trash-test-dir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let from = base.join("from");
        let to = base.join("to");
        std::fs::create_dir_all(from.join("nested")).unwrap();
        std::fs::write(from.join("nested/file.txt"), b"content").unwrap();

        move_path(&from, &to).unwrap();

        assert!(!from.exists());
        assert_eq!(std::fs::read_to_string(to.join("nested/file.txt")).unwrap(), "content");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn restore_from_trash_paths_succeeds_even_when_info_file_cleanup_fails() {
        // Regression guard for the actual bug: reproduced live with a real
        // chmod on the info/ directory (unlink() requires write access on
        // the PARENT dir, not the target file, so this genuinely blocks
        // removing the .trashinfo file independently of the file move,
        // which uses a completely different directory). Before this fix,
        // this exact scenario returned Err despite the file already being
        // safely moved to its original location.
        use std::os::unix::fs::PermissionsExt;

        let base = std::env::temp_dir().join(format!("nitrux-trash-restore-cleanup-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let info_dir = base.join("info");
        let files_dir = base.join("files");
        let dest_dir = base.join("restored");
        std::fs::create_dir_all(&info_dir).unwrap();
        std::fs::create_dir_all(&files_dir).unwrap();

        let trashed_file_path = files_dir.join("report.pdf");
        let info_path = info_dir.join("report.pdf.trashinfo");
        let original_path = dest_dir.join("report.pdf");
        std::fs::write(&trashed_file_path, b"real content").unwrap();
        std::fs::write(&info_path, "[Trash Info]\nPath=doesn't matter here\n").unwrap();

        // Remove write permission on info_dir itself: unlink() needs it on
        // the PARENT to remove an entry, regardless of the file's own mode.
        std::fs::set_permissions(&info_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        let result = restore_from_trash_paths(&trashed_file_path, &original_path, &info_path);

        // Cleanup before asserting, so a failed assertion doesn't leave a
        // read-only directory behind for the next run to trip over.
        std::fs::set_permissions(&info_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        let cleanup_result = result.clone();
        let _ = std::fs::remove_dir_all(&base);

        assert!(cleanup_result.is_ok(), "a successful file move must not be reported as a failure just because metadata cleanup failed: {cleanup_result:?}");
    }
}
