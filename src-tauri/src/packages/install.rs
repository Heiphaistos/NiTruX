use crate::subprocess;
use std::time::Duration;

/// Same allowlist the shell wrapper (`nitrux-pkexec-helper`) re-validates
/// independently — this Rust-side check is the first gate, not the only
/// one. Never remove the wrapper's own validation on the assumption this
/// makes it redundant.
pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("nom de paquet vide".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | ':' | '_' | '-')) {
        return Err(format!("nom de paquet invalide : {name}"));
    }
    Ok(())
}

pub fn validate_manager_id(manager: &str) -> Result<(), String> {
    match manager {
        "apt" | "dnf" | "pacman" | "zypper" => Ok(()),
        other => Err(format!("gestionnaire de paquets inconnu : {other}")),
    }
}

/// Installs `package` via `manager`, escalating through polkit. The
/// 5-minute timeout is deliberately generous — a real package install can
/// pull dependencies over the network, unlike every read-only command in
/// this codebase (5-20s).
#[tauri::command]
pub fn install_package(manager: String, package: String) -> Result<String, String> {
    validate_manager_id(&manager)?;
    validate_package_name(&package)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-helper", "install-package", &manager, &package],
        Duration::from_secs(300),
    )
}

/// Upgrades every detected package source. Same generous timeout
/// rationale as `install_package`, extended further since a full system
/// upgrade can be substantially slower than a single package install.
#[tauri::command]
pub fn upgrade_all_packages() -> Result<String, String> {
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-helper", "upgrade-all"],
        Duration::from_secs(1800),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_formed_package_names() {
        assert!(validate_package_name("curl").is_ok());
        assert!(validate_package_name("libssl-dev").is_ok());
        assert!(validate_package_name("python3.12").is_ok());
        assert!(validate_package_name("pkg:amd64").is_ok());
    }

    #[test]
    fn rejects_empty_package_name() {
        assert!(validate_package_name("").is_err());
    }

    #[test]
    fn rejects_shell_metacharacters_in_package_name() {
        assert!(validate_package_name("curl; rm -rf /").is_err());
        assert!(validate_package_name("curl && echo pwned").is_err());
        assert!(validate_package_name("curl`whoami`").is_err());
        assert!(validate_package_name("curl$(whoami)").is_err());
        assert!(validate_package_name("curl|cat").is_err());
        assert!(validate_package_name("../etc/passwd").is_err());
        assert!(validate_package_name("curl ").is_err());
    }

    #[test]
    fn accepts_known_managers() {
        assert!(validate_manager_id("apt").is_ok());
        assert!(validate_manager_id("dnf").is_ok());
        assert!(validate_manager_id("pacman").is_ok());
        assert!(validate_manager_id("zypper").is_ok());
    }

    #[test]
    fn rejects_unknown_manager() {
        assert!(validate_manager_id("bogus").is_err());
        assert!(validate_manager_id("").is_err());
        assert!(validate_manager_id("apt; rm -rf /").is_err());
    }
}
