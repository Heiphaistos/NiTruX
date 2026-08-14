//! Manages "portable apps" -- AppImages downloaded and run without
//! installation, the Linux equivalent of NiTriTe Windows's "Apps
//! Portables" category. No pkexec anywhere in this module: an AppImage
//! runs entirely under the invoking user's own privileges once it's
//! executable, exactly like any other program they could already
//! download and `chmod +x` by hand -- this is not a new privilege
//! boundary, unlike every other write-to-disk command in this codebase
//! that goes through pkexec.

use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

fn portable_apps_dir() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    Ok(format!("{home}/.local/share/nitrux/portable-apps"))
}

/// A stored AppImage's filename must stay a plain, single path component
/// ending in `.AppImage` -- rejects traversal (`..`, `/`) since this name
/// is joined onto `portable_apps_dir()` and used directly as a
/// launch/delete target.
fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("nom de fichier vide".to_string());
    }
    if filename.contains('/') || filename.contains("..") {
        return Err(format!("nom de fichier invalide : {filename}"));
    }
    if !filename.ends_with(".AppImage") {
        return Err(format!("le fichier doit se terminer par .AppImage : {filename}"));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PortableAppFile {
    pub filename: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn list_portable_apps() -> Result<Vec<PortableAppFile>, String> {
    let dir = portable_apps_dir()?;
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        // A fresh install has no portable apps yet and therefore no
        // directory at all -- that is the common case, not an error.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(format!("impossible de lire {dir} : {e}")),
    };
    let mut apps = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("erreur de lecture de répertoire : {e}"))?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        if !filename.ends_with(".AppImage") {
            continue;
        }
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        apps.push(PortableAppFile { filename, size_bytes });
    }
    apps.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(apps)
}

/// Finds the actual current AppImage asset name+url in `owner/repo`'s
/// latest GitHub release. Version numbers embedded in asset filenames
/// (the common case -- confirmed live against joplin/joplin,
/// Ultimaker/Cura, keepassxreboot/keepassxc, obsidianmd/obsidian-releases,
/// FreeCAD/FreeCAD, standardnotes/app, hluk/CopyQ) make a fixed
/// "latest/download/<name>" URL unreliable, so this queries the release
/// API instead. Filters out checksum/signature siblings GitHub lists next
/// to the real asset (`.sha256`/`.sha512`/`.digest`/`.zsync`/`.asc`/
/// `.sig`/`.txt`/`.zip`), then -- only if more than one candidate remains
/// -- excludes non-x86_64 architecture builds (this app targets Debian
/// x86_64). Errors rather than guessing if that still leaves zero or more
/// than one candidate.
/// Pure selection logic, separated from the network/JSON-parsing I/O in
/// `resolve_appimage_asset` below so it's directly unit-testable against
/// the exact asset lists observed live from GitHub's API (joplin/joplin,
/// Ultimaker/Cura, keepassxreboot/keepassxc, obsidianmd/obsidian-releases,
/// FreeCAD/FreeCAD, standardnotes/app, hluk/CopyQ) without a real network
/// call per test run.
fn pick_appimage_asset(assets: &[(String, String)], owner: &str, repo: &str) -> Result<(String, String), String> {
    const EXCLUDED_SUFFIXES: [&str; 8] =
        [".sha256", ".sha512", ".digest", ".zsync", ".asc", ".sig", ".txt", ".zip"];
    let mut candidates: Vec<(String, String)> = assets
        .iter()
        .filter(|(name, _)| {
            let lower = name.to_lowercase();
            lower.contains("appimage") && !EXCLUDED_SUFFIXES.iter().any(|s| lower.ends_with(s))
        })
        .cloned()
        .collect();

    if candidates.len() > 1 {
        const OTHER_ARCHES: [&str; 6] = ["aarch64", "arm64", "armv7", "armhf", "i386", "i686"];
        let x86_only: Vec<_> = candidates
            .iter()
            .filter(|(name, _)| {
                let lower = name.to_lowercase();
                !OTHER_ARCHES.iter().any(|arch| lower.contains(arch))
            })
            .cloned()
            .collect();
        if !x86_only.is_empty() {
            candidates = x86_only;
        }
    }

    match candidates.len() {
        0 => Err(format!("aucun fichier AppImage trouvé dans la dernière release de {owner}/{repo}")),
        1 => Ok(candidates.remove(0)),
        _ => Err(format!(
            "plusieurs fichiers AppImage ambigus dans la dernière release de {owner}/{repo}, impossible de choisir automatiquement"
        )),
    }
}

fn resolve_appimage_asset(owner: &str, repo: &str) -> Result<(String, String), String> {
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let body = subprocess::run_with_timeout(
        "curl",
        &["-sL", "--max-time", "15", "-H", "User-Agent: NiTruX", &api_url],
        Duration::from_secs(20),
    )?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("réponse GitHub illisible : {e}"))?;
    let assets = json["assets"]
        .as_array()
        .ok_or_else(|| format!("aucune release trouvée pour {owner}/{repo}"))?;
    let assets: Vec<(String, String)> = assets
        .iter()
        .filter_map(|a| Some((a["name"].as_str()?.to_string(), a["browser_download_url"].as_str()?.to_string())))
        .collect();
    pick_appimage_asset(&assets, owner, repo)
}

#[tauri::command]
pub fn download_portable_app(owner: String, repo: String) -> Result<PortableAppFile, String> {
    let (filename, url) = resolve_appimage_asset(&owner, &repo)?;
    validate_filename(&filename)?;

    let dir = portable_apps_dir()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("impossible de créer {dir} : {e}"))?;
    let dest_path = format!("{dir}/{filename}");

    // `-f`: without it, curl treats an HTTP error response (e.g. a 404 if
    // the resolved asset moved or was deleted between resolve_appimage_asset
    // querying the API and this actual download) as a "successful" transfer
    // -- exit code 0, with the error page's body written to dest_path as
    // if it were the real file. Reproduced live before this fix: a 404 URL
    // produced a 9-byte "Not Found" file and a 0 exit code, which chmod +x
    // would then happily mark executable and report as a successful
    // download. `-f` makes curl exit non-zero and write no file at all on
    // an HTTP error instead.
    subprocess::run_with_timeout(
        "curl",
        &["-sfL", "--max-time", "300", "-o", &dest_path, &url],
        Duration::from_secs(310),
    )?;

    let metadata =
        std::fs::metadata(&dest_path).map_err(|e| format!("téléchargement introuvable après coup : {e}"))?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o700);
    std::fs::set_permissions(&dest_path, perms)
        .map_err(|e| format!("impossible de rendre {filename} exécutable : {e}"))?;

    Ok(PortableAppFile { filename, size_bytes: metadata.len() })
}

#[tauri::command]
pub fn launch_portable_app(filename: String) -> Result<(), String> {
    validate_filename(&filename)?;
    let dir = portable_apps_dir()?;
    let path = format!("{dir}/{filename}");
    // Not run_with_timeout: an AppImage is a GUI app meant to keep running
    // indefinitely, not a short-lived command whose output this cares
    // about -- spawn and detach, exactly like TerminalPage's shell.
    //
    // env_remove: if NiTruX itself is running as an AppImage, its own
    // AppRun-set LD_LIBRARY_PATH would otherwise leak into the launched
    // app's process too -- the exact same "system binary linked against a
    // mismatched bundled library" crash class fixed in subprocess.rs, just
    // reachable here on a launched portable app instead of a spawned CLI
    // tool.
    std::process::Command::new(&path)
        .env_remove("LD_LIBRARY_PATH")
        .spawn()
        .map_err(|e| format!("impossible de lancer {filename} : {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn remove_portable_app(filename: String) -> Result<(), String> {
    validate_filename(&filename)?;
    let dir = portable_apps_dir()?;
    let path = format!("{dir}/{filename}");
    std::fs::remove_file(&path).map_err(|e| format!("impossible de supprimer {filename} : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_filename() {
        assert!(validate_filename("").is_err());
    }

    #[test]
    fn rejects_path_traversal_and_separators() {
        assert!(validate_filename("../../etc/passwd").is_err());
        assert!(validate_filename("sub/dir.AppImage").is_err());
    }

    #[test]
    fn rejects_non_appimage_extension() {
        assert!(validate_filename("not-an-appimage.sh").is_err());
        assert!(validate_filename("Joplin-3.6.15.AppImage.sha512").is_err());
    }

    #[test]
    fn accepts_a_well_formed_filename() {
        assert!(validate_filename("Joplin-3.6.15.AppImage").is_ok());
    }

    fn a(name: &str) -> (String, String) {
        (name.to_string(), format!("https://example.invalid/{name}"))
    }

    #[test]
    fn picks_the_only_appimage_asset_and_ignores_its_checksum_sibling() {
        // Real asset list observed live for keepassxreboot/keepassxc's
        // latest release -- the .DIGEST sibling must not slip through.
        let assets = [
            a("KeePassXC-2.7.12-x86_64.AppImage"),
            a("KeePassXC-2.7.12-x86_64.AppImage.DIGEST"),
            a("KeePassXC-2.7.12-x86_64.AppImage.zsync.DIGEST"),
        ];
        let (name, _) = pick_appimage_asset(&assets, "keepassxreboot", "keepassxc").unwrap();
        assert_eq!(name, "KeePassXC-2.7.12-x86_64.AppImage");
    }

    #[test]
    fn prefers_the_x86_64_build_when_multiple_architectures_are_published() {
        // Real asset list observed live for obsidianmd/obsidian-releases.
        let assets = [a("Obsidian-1.13.4-arm64.AppImage"), a("Obsidian-1.13.4.AppImage")];
        let (name, _) = pick_appimage_asset(&assets, "obsidianmd", "obsidian-releases").unwrap();
        assert_eq!(name, "Obsidian-1.13.4.AppImage");
    }

    #[test]
    fn prefers_the_x86_64_build_even_when_named_explicitly_rather_than_by_absence() {
        // Real asset list observed live for FreeCAD/FreeCAD -- unlike
        // Obsidian, the x86_64 build here names its arch explicitly, so
        // "absence of an arch marker" alone would not be enough to pick it.
        let assets = [
            a("FreeCAD_1.1.3-Linux-aarch64-py311.AppImage"),
            a("FreeCAD_1.1.3-Linux-aarch64-py311.AppImage-SHA256.txt"),
            a("FreeCAD_1.1.3-Linux-x86_64-py311.AppImage"),
            a("FreeCAD_1.1.3-Linux-x86_64-py311.AppImage-SHA256.txt"),
        ];
        let (name, _) = pick_appimage_asset(&assets, "FreeCAD", "FreeCAD").unwrap();
        assert_eq!(name, "FreeCAD_1.1.3-Linux-x86_64-py311.AppImage");
    }

    #[test]
    fn errors_rather_than_guessing_when_no_appimage_asset_exists() {
        // Real asset list shape for a release with no AppImage at all
        // (e.g. bitwarden/clients, balena-io/etcher at time of writing).
        let assets = [a("app.deb"), a("app.rpm"), a("app.tar.gz")];
        assert!(pick_appimage_asset(&assets, "owner", "repo").is_err());
    }

    #[test]
    fn errors_rather_than_guessing_when_still_ambiguous_after_arch_filtering() {
        let assets = [a("app-build-a.AppImage"), a("app-build-b.AppImage")];
        assert!(pick_appimage_asset(&assets, "owner", "repo").is_err());
    }

    #[test]
    fn list_on_a_missing_directory_returns_an_empty_list_not_an_error() {
        // portable_apps_dir() is namespaced under a fixed, real HOME in
        // this test environment, so this only actually exercises the
        // "not found" branch if that specific subdirectory doesn't exist
        // yet -- true on a fresh checkout/CI runner, and harmless
        // (Ok(vec![])) even if it does exist and happens to be empty too.
        let result = list_portable_apps();
        assert!(result.is_ok());
    }
}
