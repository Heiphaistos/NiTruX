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

use crate::secure_temp::write_exclusively_owner_only;
use crate::subprocess;
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
/// `tauri.conf.json`'s `deb`/`rpm` file maps list the same 15 names for
/// the package-manager-driven install path.
///
/// `nitrux-pkexec-delete-snapshot` (added alongside RestorePointsPage's
/// snapshot deletion, cycle 379) was missing from this list entirely --
/// this doesn't affect .deb/.rpm installs (their package manager places
/// the binary via tauri.conf.json's own file map, independent of this
/// module), but an AppImage install -- which relies solely on THIS
/// bootstrap list to create every /usr/bin/nitrux-pkexec-* binary --
/// never got the file at all, silently breaking snapshot deletion for
/// every AppImage user specifically while working fine everywhere else.
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
    "nitrux-pkexec-delete-snapshot",
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

/// Stages a plain-filesystem copy of the resources `build_bootstrap_script`
/// needs, before elevating. Required because `resource_dir` itself may be
/// the AppImage's FUSE-mounted resource path: that mount is readable only
/// by the user who launched the AppImage (AppImages don't set FUSE's
/// `allow_other`), so `pkexec`'s root-run `cp` cannot read through it at
/// all -- reproduced live on the shipped v0.25.142 AppImage: `cp: impossible
/// d'évaluer '/tmp/.mount_Nitrux.../packaging/nitrux-pkexec-helper':
/// Permission non accordée`, which then cascaded into every other
/// pkexec-routed command failing too (the binaries this step should have
/// created were simply never written). A `.deb`/`.rpm` install's
/// `resource_dir` is already a normal on-disk path readable by root, so this
/// staging step there is just a harmless extra copy, not a fix for anything.
///
/// Each file is written via `write_exclusively_owner_only` rather than
/// `std::fs::copy`, for the same predictable-temp-path reason documented in
/// `secure_temp`'s module doc (first found in this very file): a
/// pre-positioned symlink at the staged destination must not be followed.
fn stage_resources_for_pkexec(resource_dir: &Path) -> Result<PathBuf, String> {
    let staging_dir = std::env::temp_dir().join(format!("nitrux-pkexec-stage-{}", std::process::id()));
    let packaging_dir = staging_dir.join("packaging");
    std::fs::create_dir_all(&packaging_dir)
        .map_err(|e| format!("impossible de créer le dossier de préparation : {e}"))?;

    let mut names: Vec<&str> = vec!["nitrux-pkexec-helper"];
    names.extend_from_slice(POLKIT_POLICY_FILES);
    for name in names {
        let content = std::fs::read(resource_dir.join("packaging").join(name))
            .map_err(|e| format!("impossible de lire {name} dans les ressources de l'application : {e}"))?;
        write_exclusively_owner_only(&packaging_dir.join(name), &content)?;
    }
    Ok(staging_dir)
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

    let staging_dir = stage_resources_for_pkexec(&resource_dir)?;
    let script_path = write_bootstrap_script_to_temp()?;
    let result = subprocess::run_with_timeout(
        "pkexec",
        &["sh", &script_path.to_string_lossy(), &staging_dir.to_string_lossy()],
        Duration::from_secs(60),
    );
    let _ = std::fs::remove_file(&script_path);
    let _ = std::fs::remove_dir_all(&staging_dir);
    result.map(|_| "Intégration système installée avec succès.".to_string())
}

/// The script is executed as root moments later via `pkexec sh <path>` --
/// see `secure_temp::write_exclusively_owner_only` for why this can't be a
/// plain `std::fs::write` to a predictable path.
fn write_bootstrap_script_to_temp() -> Result<PathBuf, String> {
    let script_path = std::env::temp_dir().join(format!("nitrux-pkexec-bootstrap-{}.sh", std::process::id()));
    write_exclusively_owner_only(&script_path, build_bootstrap_script().as_bytes())?;
    Ok(script_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_every_resource_file_into_a_plain_readable_copy() {
        // Regression guard for the actual bug (v0.25.142 AppImage, reported
        // live): passing resource_dir straight through to the root-run
        // script failed with "Permission non accordée" because root cannot
        // read into the AppImage's per-user FUSE mount at all. Staging a
        // plain-filesystem copy first is the fix -- this proves every file
        // the bootstrap script needs actually lands in the staging dir with
        // its real content, not just that the function returns Ok.
        let resource_dir = std::env::temp_dir().join(format!("nitrux-pkexec-stage-test-src-{}", std::process::id()));
        let packaging_dir = resource_dir.join("packaging");
        std::fs::create_dir_all(&packaging_dir).unwrap();
        std::fs::write(packaging_dir.join("nitrux-pkexec-helper"), b"#!/bin/sh\necho helper\n").unwrap();
        for policy in POLKIT_POLICY_FILES {
            std::fs::write(packaging_dir.join(policy), b"<policyconfig/>").unwrap();
        }

        let result = stage_resources_for_pkexec(&resource_dir);
        std::fs::remove_dir_all(&resource_dir).ok();

        let staging_dir = result.expect("staging should succeed when every resource file is present");
        let staged_helper = std::fs::read(staging_dir.join("packaging").join("nitrux-pkexec-helper"))
            .expect("staged helper script should be readable");
        assert_eq!(staged_helper, b"#!/bin/sh\necho helper\n");
        for policy in POLKIT_POLICY_FILES {
            assert!(staging_dir.join("packaging").join(policy).exists(), "staged copy of {policy} should exist");
        }
        std::fs::remove_dir_all(&staging_dir).ok();
    }

    #[test]
    fn stage_resources_for_pkexec_surfaces_a_clear_error_when_a_resource_file_is_missing() {
        let resource_dir = std::env::temp_dir().join(format!("nitrux-pkexec-stage-test-missing-{}", std::process::id()));
        std::fs::create_dir_all(resource_dir.join("packaging")).unwrap();
        // No files written -- resource_dir exists but is empty.

        let err = stage_resources_for_pkexec(&resource_dir).expect_err("should fail when a resource file is absent");
        std::fs::remove_dir_all(&resource_dir).ok();

        assert!(err.contains("nitrux-pkexec-helper"), "error should name the missing file: {err}");
    }

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
    fn includes_delete_snapshot_alongside_create_snapshot() {
        // Regression guard for the actual bug: this name was missing
        // entirely -- the generic loop-based tests below would have
        // passed either way (they only check whatever IS in the list),
        // so this pins the specific name against an accidental future
        // removal the same way installProfiles.spec.ts pins each new
        // catalog profile.
        assert!(PKEXEC_BINARY_NAMES.contains(&"nitrux-pkexec-create-snapshot"));
        assert!(PKEXEC_BINARY_NAMES.contains(&"nitrux-pkexec-delete-snapshot"));
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

    // Exclusive-create/owner-only-permissions/symlink-preemption behavior
    // is generic and now covered once in secure_temp.rs's own tests
    // (shared by every caller of write_exclusively_owner_only) -- no
    // need to duplicate that coverage here.
}
