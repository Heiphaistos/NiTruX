//! How this copy of NiTruX was installed, and on which distribution.
//!
//! The Arch/Omarchy package repackages the .deb, whose binary tells the
//! Tauri updater "I am a deb": left alone, the updater would download a
//! .deb and run `dpkg -i` on Arch. The package drops a marker file instead,
//! and the frontend leaves updates to pacman when it finds one.

use serde::Serialize;

const MANAGED_BY_MARKER: &str = "/usr/lib/nitrux/managed-by";

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct InstallInfo {
    /// "pacman" when a system package manager owns NiTruX and updates it.
    pub managed_by: Option<String>,
    /// os-release `ID` ("omarchy", "arch", "debian"...).
    pub distro_id: String,
    /// os-release `ID_LIKE`, empty when absent.
    pub distro_like: String,
}

fn os_release_value(content: &str, key: &str) -> String {
    content
        .lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix('='))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_default()
}

fn build(os_release: &str, marker: Option<String>) -> InstallInfo {
    InstallInfo {
        managed_by: marker.map(|m| m.trim().to_string()).filter(|m| !m.is_empty()),
        distro_id: os_release_value(os_release, "ID"),
        distro_like: os_release_value(os_release, "ID_LIKE"),
    }
}

#[tauri::command]
pub fn get_install_info() -> InstallInfo {
    let os_release = std::fs::read_to_string("/etc/os-release")
        .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();
    build(&os_release, std::fs::read_to_string(MANAGED_BY_MARKER).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_omarchy_and_the_pacman_marker() {
        let info = build("NAME=\"Omarchy\"\nID=omarchy\nID_LIKE=arch\n", Some("pacman\n".into()));
        assert_eq!(
            info,
            InstallInfo { managed_by: Some("pacman".into()), distro_id: "omarchy".into(), distro_like: "arch".into() }
        );
    }

    #[test]
    fn id_is_not_confused_with_id_like_or_version_id() {
        let info = build("VERSION_ID=\"13\"\nID_LIKE=\"debian\"\nID=ubuntu\n", None);
        assert_eq!(info.distro_id, "ubuntu");
        assert_eq!(info.distro_like, "debian");
        assert_eq!(info.managed_by, None);
    }
}
