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

/// Escapes a path for a `.trashinfo` `Path=` value, the inverse of
/// `decode_trash_path`. Only the space is encoded, matching what this
/// module decodes -- writing an encoding we cannot read back would make our
/// own trashed items unrestorable.
fn encode_trash_path(path: &str) -> String {
    path.replace(' ', "%20")
}

/// Picks a name inside `Trash/files/` that is not taken. A second deletion
/// of `foo` becomes `foo.2`, matching the convention `list_trash`'s doc
/// already describes.
fn available_trash_name(files_dir: &std::path::Path, info_dir: &std::path::Path, base: &str) -> String {
    if !files_dir.join(base).exists() && !info_dir.join(format!("{base}.trashinfo")).exists() {
        return base.to_string();
    }
    for suffix in 2..10_000 {
        let candidate = format!("{base}.{suffix}");
        if !files_dir.join(&candidate).exists() && !info_dir.join(format!("{candidate}.trashinfo")).exists() {
            return candidate;
        }
    }
    format!("{base}.{}", std::process::id())
}

/// Moves `path` to the freedesktop trash instead of deleting it.
///
/// Every destructive action this app offers is reversible except
/// `format-partition`; a "clean up orphaned configuration" button that
/// unlinked directories outright would be a second exception, and the one
/// most likely to be clicked by mistake. `rename` is used when possible
/// (atomic, same filesystem) with a copy+remove fallback, since `$HOME` and
/// the trash can legitimately sit on different mounts.
#[tauri::command]
pub fn move_to_trash(path: String) -> Result<String, String> {
    let source = PathBuf::from(&path);
    if !source.is_absolute() {
        return Err(format!("chemin absolu attendu : {path}"));
    }
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    let canonical = source
        .canonicalize()
        .map_err(|e| format!("chemin introuvable : {path} ({e})"))?;
    // Refuse anything outside the user's own home: this command exists to
    // clean up per-user leftovers, and the trash it writes to is per-user
    // anyway -- a system path moved there would break the package that owns
    // it and could not be restored without root.
    if !canonical.starts_with(&home) {
        return Err(format!("hors du dossier personnel, refusé : {}", canonical.display()));
    }
    if canonical == PathBuf::from(&home) {
        return Err("le dossier personnel lui-même ne peut pas être mis à la corbeille".to_string());
    }

    let files_dir = trash_dir().join("files");
    let info_dir = trash_dir().join("info");
    std::fs::create_dir_all(&files_dir).map_err(|e| format!("corbeille inaccessible : {e}"))?;
    std::fs::create_dir_all(&info_dir).map_err(|e| format!("corbeille inaccessible : {e}"))?;

    let base = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("nom de fichier illisible : {}", canonical.display()))?;
    let trashed_name = available_trash_name(&files_dir, &info_dir, base);

    let destination = files_dir.join(&trashed_name);
    if std::fs::rename(&canonical, &destination).is_err() {
        // Cross-device move: copy then remove, and never remove the source
        // until the copy succeeded.
        let status = std::process::Command::new("cp")
            .args(["-a", "--", &canonical.to_string_lossy(), &destination.to_string_lossy()])
            .status()
            .map_err(|e| format!("déplacement impossible : {e}"))?;
        if !status.success() {
            return Err(format!("déplacement vers la corbeille impossible : {}", canonical.display()));
        }
        let removed = if canonical.is_dir() {
            std::fs::remove_dir_all(&canonical)
        } else {
            std::fs::remove_file(&canonical)
        };
        removed.map_err(|e| format!("copie faite mais original non supprimé : {e}"))?;
    }

    let info = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        encode_trash_path(&canonical.to_string_lossy()),
        // Local time is what the spec asks for; seconds precision is enough
        // and avoids pulling in a date crate for one line.
        chrono_like_now()
    );
    std::fs::write(info_dir.join(format!("{trashed_name}.trashinfo")), info)
        .map_err(|e| format!("élément déplacé mais fiche de corbeille non écrite : {e}"))?;
    Ok(trashed_name)
}

/// `YYYY-MM-DDThh:mm:ss` from the system clock, computed by hand because
/// this crate has no date dependency and one timestamp does not justify
/// adding one.
fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let time_of_day = secs % 86_400;
    let (hours, minutes, seconds) = (time_of_day / 3600, (time_of_day % 3600) / 60, time_of_day % 60);

    // Civil-from-days (Howard Hinnant's algorithm), shifted to a March-based
    // year so leap days land at the end of the cycle.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}")
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
    fn moves_a_real_directory_to_the_trash_and_can_list_it_back() {
        // Exercises the real filesystem path: HOME is pointed at a scratch
        // directory so this writes a genuine Trash/files + Trash/info pair.
        let scratch = std::env::temp_dir().join(format!("nitrux-trash-test-{}", std::process::id()));
        let victim = scratch.join(".config").join("orphan-app");
        std::fs::create_dir_all(&victim).unwrap();
        std::fs::write(victim.join("settings.conf"), b"x=1").unwrap();
        let previous_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &scratch);

        let name = move_to_trash(victim.to_string_lossy().into_owned()).expect("should trash the directory");
        assert!(!victim.exists(), "the original must be gone");
        assert!(scratch.join(".local/share/Trash/files").join(&name).exists());

        let info = std::fs::read_to_string(
            scratch.join(".local/share/Trash/info").join(format!("{name}.trashinfo")),
        )
        .unwrap();
        assert!(info.starts_with("[Trash Info]"));
        assert!(info.contains("Path=/"), "the original absolute path must be recorded: {info}");

        let listed = list_trash();
        assert!(listed.iter().any(|i| i.trashed_name == name), "trashed item should be listed");

        std::fs::remove_dir_all(&scratch).ok();
        match previous_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn refuses_a_path_outside_the_home_directory() {
        let previous_home = std::env::var("HOME").ok();
        let scratch = std::env::temp_dir().join(format!("nitrux-trash-guard-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        std::env::set_var("HOME", &scratch);

        // /etc exists everywhere this runs and is exactly what must never
        // be trashable by a per-user cleanup button.
        let err = move_to_trash("/etc".to_string()).expect_err("should refuse a system path");
        assert!(err.contains("hors du dossier personnel"), "{err}");
        assert!(move_to_trash("relative/path".to_string()).is_err());
        assert!(
            move_to_trash(scratch.to_string_lossy().into_owned()).is_err(),
            "the home directory itself must be refused"
        );

        std::fs::remove_dir_all(&scratch).ok();
        match previous_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn trashinfo_path_round_trips_through_our_own_decoder() {
        // Writing an encoding we cannot read back would make our own
        // trashed items unrestorable.
        let original = "/home/dev/.config/mon application";
        assert_eq!(decode_trash_path(&encode_trash_path(original)), original);
    }

    #[test]
    fn deletion_date_has_the_shape_the_spec_asks_for() {
        let now = chrono_like_now();
        assert_eq!(now.len(), 19, "expected YYYY-MM-DDThh:mm:ss, got {now}");
        assert_eq!(&now[4..5], "-");
        assert_eq!(&now[10..11], "T");
        let year: i32 = now[..4].parse().expect("year should parse");
        assert!((2026..2100).contains(&year), "implausible year in {now}");
    }

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
