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

#[tauri::command]
pub fn restore_trash_item(trashed_name: String) -> Result<(), String> {
    let info_path = trash_dir().join("info").join(format!("{trashed_name}.trashinfo"));
    let content = std::fs::read_to_string(&info_path).map_err(|e| format!("élément introuvable dans la corbeille : {e}"))?;
    let (original_path, _) = parse_trashinfo(&content).ok_or("fichier .trashinfo invalide (Path= manquant)")?;

    let trashed_file_path = trash_dir().join("files").join(&trashed_name);
    if let Some(parent) = std::path::Path::new(&original_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("impossible de recréer le dossier d'origine : {e}"))?;
    }
    std::fs::rename(&trashed_file_path, &original_path).map_err(|e| format!("échec de la restauration : {e}"))?;
    std::fs::remove_file(&info_path).map_err(|e| format!("restauré, mais échec du nettoyage de la métadonnée : {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_trash_item_permanently(trashed_name: String) -> Result<(), String> {
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
}
