use serde::Serialize;

pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod universal;
pub mod zypper;

#[derive(Serialize, Clone)]
pub struct PackageUpdate {
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    /// Which package manager reported this update ("apt", "dnf", "pacman",
    /// "zypper", "flatpak", "snap") — lets the frontend group/badge origin
    /// without needing to know anything else about the backend.
    pub source: String,
}

pub trait PackageManager {
    fn id(&self) -> &'static str;
    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String>;
}

/// True if `binary` is found on PATH (via `which`), false otherwise —
/// including if `which` itself is missing, which just means "not found".
pub fn binary_exists(binary: &str) -> bool {
    std::process::Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
    managers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detected_manager_id_matches_binary_name() {
        // Detection depends on the actual host's installed binaries, so this
        // test only asserts internal consistency: whatever IS detected must
        // report an id from the known set, never an empty/garbage string.
        let known_ids = ["apt", "dnf", "pacman", "zypper"];
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
