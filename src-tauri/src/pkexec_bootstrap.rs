//! First-launch self-install of the privileged-action integration for
//! packaging formats that cannot run a post-install step -- specifically
//! AppImage. `.deb`/`.rpm` installs already place the pkexec helper
//! binaries and polkit policy files via `tauri.conf.json`'s
//! `bundle.linux.deb/rpm.files` (their package managers run that as part
//! of installation); AppImage has no equivalent mechanism at all -- it is
//! a self-contained, unprivileged executable that cannot write to system
//! paths on its own. Without this module, every pkexec-routed command in
//! the app (install/uninstall packages, troubleshoot actions, hosts/DNS,
//! firewall rules, disk formatting, snapshots, quarantine) fails on an
//! AppImage install with "No such file or directory", because
//! `/usr/bin/nitrux-pkexec-*` was never created.
//!
//! This module bundles the same helper script and policy files as app
//! *resources* (bundled inside every package format, including AppImage --
//! see `tauri.conf.json`'s `bundle.resources`), and offers a one-time,
//! user-triggered `pkexec`-gated copy of them into place.

use crate::subprocess;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::Manager;

/// Every `/usr/bin/nitrux-pkexec-*` binary name the app dispatches
/// through, each a separate on-disk copy of the same
/// `nitrux-pkexec-helper` script content (see that script's own header
/// comment: pkexec resolves which action to authorize purely by matching
/// the invoked executable's path, with no visibility into argv, so a
/// shared path across multiple actions is ambiguous to it -- confirmed
/// live on a real polkit stack). Kept as a single source of truth here;
/// `tauri.conf.json`'s `deb`/`rpm` file maps list the same 14 names for
/// the package-manager-driven install path.
pub const PKEXEC_BINARY_NAMES: &[&str] = &[
    "nitrux-pkexec-install-package",
    "nitrux-pkexec-install-snap",
    "nitrux-pkexec-system-tools",
    "nitrux-pkexec-uninstall-package",
    "nitrux-pkexec-upgrade-all",
    "nitrux-pkexec-write-hosts",
    "nitrux-pkexec-set-dns",
    "nitrux-pkexec-firewall-rule",
    "nitrux-pkexec-troubleshoot",
    "nitrux-pkexec-create-snapshot",
    "nitrux-pkexec-quarantine-file",
    "nitrux-pkexec-format-partition",
    "nitrux-pkexec-extend-partition",
    "nitrux-pkexec-clone-disk",
];

/// The polkit action policy files that must be installed under
/// `/usr/share/polkit-1/actions/` for the app's own dedicated pkexec
/// actions to show a proper, app-specific authorization prompt. Not
/// strictly required for `install_pkexec_integration` itself to run (see
/// that function's doc comment on polkit's generic fallback), but
/// required for every OTHER pkexec-routed command afterwards.
pub const POLKIT_POLICY_FILES: &[&str] = &[
    "org.heiphaistos.nitrux.packages.policy",
    "org.heiphaistos.nitrux.network.policy",
    "org.heiphaistos.nitrux.system-tools.policy",
    "org.heiphaistos.nitrux.security.policy",
    "org.heiphaistos.nitrux.disks.policy",
];

/// `/usr/bin/nitrux-pkexec-troubleshoot` specifically is used as the
/// single representative check for "is the integration installed at
/// all": all binaries and policies are installed atomically by the same
/// mechanism (either the .deb/rpm package manager, or
/// `install_pkexec_integration` below), so a partially-installed state is
/// not a realistic condition this needs to detect separately.
fn is_installed_at(representative_binary: &Path) -> bool {
    representative_binary.exists()
}

#[tauri::command]
pub fn is_pkexec_integration_installed() -> bool {
    is_installed_at(Path::new("/usr/bin/nitrux-pkexec-troubleshoot"))
}

/// Builds the bootstrap script run under `pkexec`. `resource_dir` is
/// passed to the script as `$1` at invocation time (see
/// `install_pkexec_integration`), never interpolated into the script body
/// itself -- this function's output is identical on every call regardless
/// of the actual resource directory, so there is no path-injection
/// surface in the generated script text.
fn build_bootstrap_script() -> String {
    let mut script = String::from("#!/bin/sh\nset -eu\nRESOURCE_DIR=\"$1\"\n");
    for name in PKEXEC_BINARY_NAMES {
        script.push_str(&format!(
            "cp \"$RESOURCE_DIR/packaging/nitrux-pkexec-helper\" \"/usr/bin/{name}\"\nchmod 755 \"/usr/bin/{name}\"\n"
        ));
    }
    for policy in POLKIT_POLICY_FILES {
        script.push_str(&format!(
            "cp \"$RESOURCE_DIR/packaging/{policy}\" \"/usr/share/polkit-1/actions/{policy}\"\nchmod 644 \"/usr/share/polkit-1/actions/{policy}\"\n"
        ));
    }
    script
}

/// Installs the privileged-action integration on a system where it isn't
/// present yet (the AppImage bootstrap path -- see this module's own doc
/// comment). Relies on polkit's own built-in `org.freedesktop.policykit.exec`
/// fallback authorization: `pkexec <program>` works even when no
/// app-specific policy exists for `<program>` yet, prompting for admin
/// authentication via polkit's default generic rule -- this is the exact
/// mechanism that makes plain `pkexec whoami` work out of the box on any
/// system with polkit installed, not something specific to this app. That
/// resolves the apparent chicken-and-egg problem: this bootstrap step
/// needs no pre-existing NiTruX-specific policy to run, even though it is
/// the very thing that installs those policies for every other command.
#[tauri::command]
pub fn install_pkexec_integration(app: tauri::AppHandle) -> Result<String, String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("impossible de localiser le dossier de ressources de l'application : {e}"))?;

    // Defense in depth: resource_dir comes from Tauri's own trusted API,
    // never from user input, but it is still a value fed into a
    // privileged (pkexec) invocation below -- reject anything that isn't
    // a real, absolute, currently-existing directory before ever
    // shelling out, mirroring this codebase's established validate-first
    // discipline for every other pkexec-adjacent command.
    if !resource_dir.is_absolute() || !resource_dir.is_dir() {
        return Err(format!("dossier de ressources invalide : {}", resource_dir.display()));
    }

    let script_path = write_bootstrap_script_to_temp()?;
    let result = subprocess::run_with_timeout(
        "pkexec",
        &["sh", &script_path.to_string_lossy(), &resource_dir.to_string_lossy()],
        Duration::from_secs(60),
    );
    let _ = std::fs::remove_file(&script_path);
    result.map(|_| "Intégration système installée avec succès.".to_string())
}

/// Writes `content` to `path`, but never via a plain overwrite:
/// `create_new` makes the open fail if ANYTHING already exists at that
/// path (including a symlink an attacker planted there in advance,
/// pointing wherever they like) instead of silently following it, and
/// `mode(0o600)` makes the file readable and writable only by the user
/// running the app -- root (the identity that will actually read and
/// execute this file once the caller below hands its path to `pkexec sh`)
/// bypasses Unix file permissions entirely regardless, but no OTHER
/// unprivileged user on the system can read or tamper with it while it
/// exists. A plain `std::fs::write` (the previous implementation) has
/// neither guard: predictable name plus default (world-readable)
/// permissions plus silent symlink-following is the textbook
/// CWE-377/CWE-367 shape for a temp file that ends up executed with
/// elevated privileges. The best-effort `remove_file` first clears both a
/// stale leftover from a prior run that never reached its own cleanup,
/// and any attacker-planted symlink -- either way `create_new` right
/// after only proceeds if the path is genuinely clear at that instant.
fn write_exclusively_owner_only(path: &Path, content: &str) -> Result<(), String> {
    let _ = std::fs::remove_file(path);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| format!("impossible d'écrire le script d'installation : {e}"))?;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("impossible d'écrire le script d'installation : {e}"))
}

fn write_bootstrap_script_to_temp() -> Result<PathBuf, String> {
    let script_path = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-{}.sh", std::process::id()));
    write_exclusively_owner_only(&script_path, &build_bootstrap_script())?;
    Ok(script_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representative_binary_present_reports_installed() {
        let dir = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-test-present-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("nitrux-pkexec-troubleshoot");
        std::fs::write(&bin, "").unwrap();

        let installed = is_installed_at(&bin);
        std::fs::remove_dir_all(&dir).ok();

        assert!(installed);
    }

    #[test]
    fn representative_binary_absent_reports_not_installed() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/nitrux-pkexec-troubleshoot");
        assert!(!is_installed_at(bogus));
    }

    #[test]
    fn bootstrap_script_copies_every_pkexec_binary_name_and_chmods_it() {
        let script = build_bootstrap_script();
        assert!(script.starts_with("#!/bin/sh\nset -eu\n"));
        for name in PKEXEC_BINARY_NAMES {
            assert!(
                script.contains(&format!("cp \"$RESOURCE_DIR/packaging/nitrux-pkexec-helper\" \"/usr/bin/{name}\"")),
                "script should copy the helper to /usr/bin/{name}"
            );
            assert!(script.contains(&format!("chmod 755 \"/usr/bin/{name}\"")), "script should chmod /usr/bin/{name}");
        }
    }

    #[test]
    fn bootstrap_script_copies_every_polkit_policy_and_chmods_it() {
        let script = build_bootstrap_script();
        for policy in POLKIT_POLICY_FILES {
            assert!(
                script.contains(&format!(
                    "cp \"$RESOURCE_DIR/packaging/{policy}\" \"/usr/share/polkit-1/actions/{policy}\""
                )),
                "script should copy {policy} into polkit's actions dir"
            );
            assert!(
                script.contains(&format!("chmod 644 \"/usr/share/polkit-1/actions/{policy}\"")),
                "script should chmod {policy}"
            );
        }
    }

    #[test]
    fn bootstrap_script_is_deterministic_and_has_no_injection_surface() {
        // The script body never embeds resource_dir directly -- it reads
        // it from $1 at runtime instead -- so its content must be
        // identical across calls regardless of any external state.
        assert_eq!(build_bootstrap_script(), build_bootstrap_script());
        assert!(!build_bootstrap_script().contains("$(") && !build_bootstrap_script().contains('`'), "no command substitution should appear in the generated script");
    }

    #[test]
    fn write_bootstrap_script_to_temp_produces_a_readable_valid_shell_script() {
        let path = write_bootstrap_script_to_temp().expect("should write the script");
        let content = std::fs::read_to_string(&path).expect("script file should be readable");
        std::fs::remove_file(&path).ok();
        assert_eq!(content, build_bootstrap_script());
        assert!(content.starts_with("#!/bin/sh"));
    }

    #[test]
    fn script_file_is_created_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-test-perms-{}.sh", std::process::id()));
        write_exclusively_owner_only(&path, "#!/bin/sh\necho hi\n").expect("should write");
        let mode = std::fs::metadata(&path).expect("should stat the file").permissions().mode() & 0o777;
        std::fs::remove_file(&path).ok();
        assert_eq!(mode, 0o600, "script file must be readable/writable only by its owner, not world-readable like a plain fs::write would leave it");
    }

    #[test]
    fn a_preexisting_symlink_at_the_target_path_is_cleared_not_followed() {
        // Simulates an attacker (or a stale leftover from a prior crashed
        // run) having planted a symlink at the exact predictable path
        // before this runs, pointing at some file the calling user can
        // write to -- a plain `fs::write` would follow it and clobber
        // that file's content with the bootstrap script instead of
        // writing a fresh, independent file at the intended path.
        let canary_dir = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-test-canary-{}", std::process::id()));
        std::fs::create_dir_all(&canary_dir).unwrap();
        let canary = canary_dir.join("canary.txt");
        std::fs::write(&canary, "untouched").unwrap();

        let target_path = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-test-symlink-{}.sh", std::process::id()));
        std::os::unix::fs::symlink(&canary, &target_path).expect("should create the pre-positioned symlink");

        let result = write_exclusively_owner_only(&target_path, "#!/bin/sh\necho hi\n");

        let canary_content = std::fs::read_to_string(&canary).unwrap();
        let target_is_symlink = std::fs::symlink_metadata(&target_path).map(|m| m.file_type().is_symlink()).unwrap_or(false);
        std::fs::remove_file(&target_path).ok();
        std::fs::remove_dir_all(&canary_dir).ok();

        result.expect("should succeed despite the pre-positioned symlink");
        assert_eq!(canary_content, "untouched", "the canary file must never receive the bootstrap script content");
        assert!(!target_is_symlink, "the resulting path must be a real file, not still a symlink");
    }
}
