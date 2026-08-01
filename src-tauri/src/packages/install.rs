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
///
/// Invokes the dedicated `nitrux-pkexec-install-package` path rather than
/// a path shared with other actions: pkexec resolves which polkit action
/// to authorize purely by matching the executable path against each
/// action's exec.path annotation, with no visibility into argv. A shared
/// path across multiple actions is ambiguous to it — confirmed live on a
/// real polkit stack, where pkexec authorized under the wrong action when
/// two actions shared one path. Each action gets its own on-disk copy of
/// the same script under its own name to remove the ambiguity.
#[tauri::command]
pub fn install_package(manager: String, package: String) -> Result<String, String> {
    validate_manager_id(&manager)?;
    validate_package_name(&package)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-install-package", "install-package", &manager, &package],
        Duration::from_secs(300),
    )
}

/// Uninstalls `package` via `manager`, escalating through polkit. Mirrors
/// `install_package` exactly (same validation, same generous timeout for a
/// potentially dependency-heavy operation) -- see that function's doc
/// comment for why this uses its own dedicated `nitrux-pkexec-uninstall-package`
/// exec path rather than sharing `install-package`'s.
#[tauri::command]
pub fn uninstall_package(manager: String, package: String) -> Result<String, String> {
    validate_manager_id(&manager)?;
    validate_package_name(&package)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-uninstall-package", "uninstall-package", &manager, &package],
        Duration::from_secs(300),
    )
}

/// Upgrades every detected package source. Same generous timeout
/// rationale as `install_package`, extended further since a full system
/// upgrade can be substantially slower than a single package install.
/// See `install_package`'s doc comment for why this uses its own
/// dedicated `nitrux-pkexec-upgrade-all` exec path.
#[tauri::command]
pub fn upgrade_all_packages() -> Result<String, String> {
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-upgrade-all", "upgrade-all"],
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

    #[test]
    fn uninstall_package_rejects_unknown_manager_before_ever_shelling_out() {
        let result = uninstall_package("bogus-manager".to_string(), "curl".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_package_rejects_malicious_package_name_before_ever_shelling_out() {
        let result = uninstall_package("apt".to_string(), "curl; rm -rf /".to_string());
        assert!(result.is_err());
    }
}
