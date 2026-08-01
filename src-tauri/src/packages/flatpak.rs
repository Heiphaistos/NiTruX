use crate::packages::binary_exists;
use crate::packages::install::validate_package_name;
use crate::subprocess;
use std::time::Duration;

const FLATHUB_REPO_URL: &str = "https://flathub.org/repo/flathub.flatpakrepo";

/// Installs `app_id` (a reverse-DNS Flatpak application id, e.g.
/// "com.discordapp.Discord") in `--user` scope. Deliberately NOT routed
/// through pkexec: `flatpak install --user` writes to
/// `~/.local/share/flatpak`, under the invoking user's own account, the
/// same "not a new privilege boundary" reasoning R8 applied to
/// `run_script`. Reuses `validate_package_name` as-is -- it already
/// accepts '.', '+', ':', '_', '-', which covers every real Flatpak app id
/// without modification.
///
/// A fresh Flatpak install has zero remotes configured (confirmed on the
/// project's dev VM), so `flatpak install` would fail before ever reaching
/// the app id -- `remote-add --if-not-exists` is run first, unconditionally
/// and idempotently, so this works on both a fresh install and one that
/// already has flathub configured.
#[tauri::command]
pub fn install_flatpak_package(app_id: String) -> Result<String, String> {
    validate_package_name(&app_id)?;
    if !binary_exists("flatpak") {
        return Err("flatpak n'est pas installé sur ce système".to_string());
    }
    // Best-effort: if this fails for a reason other than "already exists"
    // (which --if-not-exists itself already suppresses), the install
    // attempt right after will surface a clear error anyway.
    let _ = subprocess::run_with_timeout(
        "flatpak",
        &["remote-add", "--if-not-exists", "--user", "flathub", FLATHUB_REPO_URL],
        Duration::from_secs(30),
    );
    subprocess::run_with_timeout(
        "flatpak",
        &["install", "--user", "--noninteractive", "flathub", &app_id],
        Duration::from_secs(300),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_app_id() {
        assert!(install_flatpak_package("".to_string()).is_err());
    }

    #[test]
    fn rejects_app_id_with_shell_metacharacters() {
        assert!(install_flatpak_package("com.example.App; rm -rf /".to_string()).is_err());
    }

    #[test]
    fn accepts_a_well_formed_reverse_dns_app_id_through_validation() {
        // Validation is the only thing this test can exercise portably --
        // it does not actually run flatpak (would require it installed and
        // network access, neither guaranteed in a test sandbox). The real
        // install path is verified live on the VM in Task 6.
        assert!(validate_package_name("com.discordapp.Discord").is_ok());
        assert!(validate_package_name("com.valvesoftware.Steam").is_ok());
    }
}
