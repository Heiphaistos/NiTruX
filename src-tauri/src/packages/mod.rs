use serde::Serialize;

pub mod apk;
pub mod apt;
pub mod dnf;
pub mod flatpak;
pub mod install;
pub mod pacman;
pub mod store;
pub mod universal;
pub mod xbps;
pub mod zypper;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PackageUpdate {
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    /// Which package manager reported this update ("apt", "dnf", "pacman",
    /// "zypper", "flatpak", "snap") — lets the frontend group/badge origin
    /// without needing to know anything else about the backend.
    pub source: String,
}

#[derive(Serialize, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

pub trait PackageManager {
    fn id(&self) -> &'static str;
    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String>;
    fn list_installed(&self) -> Result<Vec<InstalledPackage>, String>;
}

/// True if `binary` is installed. Checked with a directory scan (PATH plus
/// the sbin directories), not by spawning `which`: `which` is absent from
/// minimal Fedora, Alpine, Arch base and many containers, where every
/// package manager then read as missing.
pub fn binary_exists(binary: &str) -> bool {
    crate::subprocess::binary_in_path(binary)
}

/// Detects which native package managers are present on this host by binary
/// presence. Multiple can be detected simultaneously (e.g. a distro that
/// ships both for historical reasons) — every detected one is queried.
pub fn detect_package_managers() -> Vec<Box<dyn PackageManager>> {
    let mut managers: Vec<Box<dyn PackageManager>> = Vec::new();
    if binary_exists("apt") {
        managers.push(Box::new(apt::Apt));
    }
    if binary_exists("dnf") {
        managers.push(Box::new(dnf::Dnf));
    }
    if binary_exists("pacman") {
        managers.push(Box::new(pacman::Pacman));
    }
    if binary_exists("zypper") {
        managers.push(Box::new(zypper::Zypper));
    }
    if binary_exists("apk") {
        managers.push(Box::new(apk::Apk));
    }
    if binary_exists("xbps-install") {
        managers.push(Box::new(xbps::Xbps));
    }
    managers
}

/// Aggregates installed packages from the FIRST detected native manager
/// only (mirrors `detect_native_manager`'s own "just the first one" choice
/// from Phase R3 -- a host with multiple native managers installed is rare
/// and install/uninstall already only ever targets one at a time via that
/// same detected id).
pub fn list_installed_for_detected_manager() -> Result<Vec<InstalledPackage>, String> {
    let managers = detect_package_managers();
    let Some(manager) = managers.first() else {
        return Err("aucun gestionnaire de paquets détecté".to_string());
    };
    manager.list_installed()
}

#[tauri::command]
pub fn list_installed_packages() -> Result<Vec<InstalledPackage>, String> {
    list_installed_for_detected_manager()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected_manager_id_matches_binary_name() {
        // Detection depends on the actual host's installed binaries, so this
        // test only asserts internal consistency: whatever IS detected must
        // report an id from the known set, never an empty/garbage string.
        let known_ids = ["apt", "dnf", "pacman", "zypper", "apk", "xbps"];
        for m in detect_package_managers() {
            assert!(known_ids.contains(&m.id()), "unexpected manager id: {}", m.id());
        }
    }

    #[test]
    fn binary_exists_returns_false_for_bogus_binary() {
        assert!(!binary_exists("definitely-not-a-real-binary-xyz"));
    }

    #[test]
    fn binary_exists_returns_true_for_a_known_present_binary() {
        // `sh` is present on every POSIX system, including minimal containers.
        assert!(binary_exists("sh"));
    }
}
