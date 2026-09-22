//! Finds per-user configuration left behind by uninstalled software.
//!
//! Uninstalling a package removes what the package manager put in `/usr`,
//! never what the application wrote into `~/.config`, `~/.local/share` or
//! `~/.cache`. Years of that accumulates, and nothing on a Linux desktop
//! reports it.
//!
//! Deciding "orphan" is a heuristic, so this module only ever *reports*.
//! The user chooses what goes, and what goes is moved to the trash by
//! `trash::move_to_trash`, never unlinked here.

use crate::subprocess;
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone)]
pub struct OrphanConfig {
    pub path: String,
    /// Directory name, which is what the user recognises ("discord",
    /// "libreoffice").
    pub name: String,
    pub size_bytes: u64,
    /// Which directory it was found in, so the UI can say whether this is
    /// configuration, data or cache.
    pub kind: String,
}

/// Names that belong to the desktop environment or to shared conventions
/// rather than to one application. Flagging these would be actively
/// dangerous: removing `~/.config/systemd` or `~/.local/share/keyrings`
/// breaks a working session, and no package "owns" them in the sense this
/// scan checks for.
const NEVER_ORPHAN: &[&str] = &[
    "autostart", "dconf", "environment.d", "fontconfig", "gtk-2.0", "gtk-3.0", "gtk-4.0",
    "ibus", "keyrings", "mime", "mimeapps.list", "nautilus", "pulse", "systemd", "user-dirs.dirs",
    "user-dirs.locale", "applications", "icons", "fonts", "themes", "backgrounds", "flatpak",
    "gnome-boxes", "gnome-control-center", "gnome-session", "gnome-shell", "kde", "kdeglobals",
    "plasma-workspace", "pipewire", "wireplumber", "xdg-desktop-portal", "Trash", "nitrux",
];

/// Everything installed that could plausibly own a config directory: every
/// executable name on `PATH`, plus every `.desktop` file's stem. A single
/// pass, reused for every candidate, instead of one lookup per directory.
pub fn installed_identifiers() -> HashSet<String> {
    let mut names = HashSet::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        names.insert(name.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let desktop_dirs = [
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from(&home).join(".local/share/applications"),
    ];
    for dir in desktop_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(stem) = entry.file_name().to_str().and_then(|n| n.strip_suffix(".desktop")) {
                    names.insert(stem.to_ascii_lowercase());
                    // Reverse-DNS ids (org.gnome.Calculator) should also
                    // match a plain `calculator` directory.
                    if let Some(last) = stem.rsplit('.').next() {
                        names.insert(last.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    names
}

/// Whether `dir_name` looks like it still belongs to something installed.
///
/// Matching is deliberately generous: a directory is only reported as an
/// orphan when nothing installed resembles it at all. `~/.config/nvim`
/// should not be flagged because the binary is `nvim`, and
/// `~/.config/Code - OSS` should not be flagged because `code` exists.
pub fn looks_installed(dir_name: &str, installed: &HashSet<String>) -> bool {
    let name = dir_name.to_ascii_lowercase();
    if installed.contains(&name) {
        return true;
    }
    // Strip the decorations applications add to their directory names.
    let simplified: String = name
        .chars()
        .take_while(|c| *c != ' ')
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if simplified.len() >= 3 && installed.contains(&simplified) {
        return true;
    }
    if let Some(last) = name.rsplit('.').next() {
        if last.len() >= 3 && installed.contains(last) {
            return true;
        }
    }
    installed
        .iter()
        .any(|candidate| candidate.len() >= 4 && (name.starts_with(candidate) || candidate.starts_with(&name) && name.len() >= 4))
}

fn directory_size(path: &Path) -> u64 {
    subprocess::run_with_timeout(
        "du",
        &["-sb", "-x", &path.to_string_lossy()],
        std::time::Duration::from_secs(20),
    )
    .ok()
    .and_then(|out| out.split_whitespace().next().and_then(|v| v.parse().ok()))
    .unwrap_or(0)
}

fn scan_dir(base: &Path, kind: &str, installed: &HashSet<String>, out: &mut Vec<OrphanConfig>) {
    let Ok(entries) = std::fs::read_dir(base) else { return };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else { continue };
        if name.starts_with('.') || NEVER_ORPHAN.contains(&name.as_str()) {
            continue;
        }
        // Only directories: a stray `.conf` file is a few hundred bytes and
        // far more likely to be something the user wrote themselves.
        if !entry.path().is_dir() {
            continue;
        }
        if looks_installed(&name, installed) {
            continue;
        }
        out.push(OrphanConfig {
            path: entry.path().to_string_lossy().into_owned(),
            name,
            size_bytes: directory_size(&entry.path()),
            kind: kind.to_string(),
        });
    }
}

#[tauri::command]
pub fn list_orphan_configs() -> Result<Vec<OrphanConfig>, String> {
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    let home = PathBuf::from(home);
    let installed = installed_identifiers();

    let mut orphans = Vec::new();
    scan_dir(&home.join(".config"), "configuration", &installed, &mut orphans);
    scan_dir(&home.join(".local/share"), "données", &installed, &mut orphans);
    scan_dir(&home.join(".cache"), "cache", &installed, &mut orphans);
    orphans.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(orphans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed_set(names: &[&str]) -> HashSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn keeps_a_directory_whose_binary_still_exists() {
        let installed = installed_set(&["nvim", "code", "firefox"]);
        assert!(looks_installed("nvim", &installed));
        assert!(looks_installed("Code - OSS", &installed), "decorated names must still match");
        assert!(looks_installed("firefox", &installed));
    }

    #[test]
    fn keeps_a_reverse_dns_directory_matching_an_installed_app() {
        let installed = installed_set(&["calculator", "gimp"]);
        assert!(looks_installed("org.gnome.Calculator", &installed));
        assert!(looks_installed("GIMP", &installed));
    }

    #[test]
    fn flags_a_directory_nothing_installed_resembles() {
        let installed = installed_set(&["nvim", "code"]);
        assert!(!looks_installed("skypeforlinux", &installed));
        assert!(!looks_installed("some-removed-app", &installed));
    }

    #[test]
    fn never_flags_desktop_environment_directories() {
        // These have no owning binary, so the heuristic alone would report
        // them -- and removing ~/.config/systemd or the keyring breaks a
        // working session.
        for name in ["systemd", "autostart", "dconf", "keyrings", "pulse", "nitrux"] {
            assert!(NEVER_ORPHAN.contains(&name), "{name} must be protected");
        }
    }

    #[test]
    fn scans_a_real_directory_and_reports_only_the_orphan() {
        let scratch = std::env::temp_dir().join(format!("nitrux-orphan-test-{}", std::process::id()));
        let config = scratch.join(".config");
        std::fs::create_dir_all(config.join("skypeforlinux")).unwrap();
        std::fs::create_dir_all(config.join("systemd")).unwrap();
        std::fs::create_dir_all(config.join("nvim")).unwrap();
        std::fs::write(config.join("skypeforlinux").join("settings.json"), vec![0u8; 4096]).unwrap();

        let installed = installed_set(&["nvim"]);
        let mut found = Vec::new();
        scan_dir(&config, "configuration", &installed, &mut found);

        let names: Vec<&str> = found.iter().map(|o| o.name.as_str()).collect();
        assert_eq!(names, vec!["skypeforlinux"], "systemd is protected, nvim is installed");
        assert!(found[0].size_bytes > 0, "size should come from a real du run");

        std::fs::remove_dir_all(&scratch).ok();
    }
}
