use crate::subprocess;
use std::time::Duration;

/// The complete, fixed set of privileged one-click actions this catalog
/// exposes. Deliberately a closed allowlist -- NEVER accept an arbitrary
/// action string and NEVER construct the underlying shell command from
/// anything but a hardcoded match arm, both here and in the wrapper script
/// this dispatches to. Each of these 7 actions was chosen because it is
/// safe, idempotent, and never touches user data (see the design spec's
/// table for the exact command each one runs).
const VALID_ACTIONS: &[&str] = &[
    "apt-autoremove",
    "journal-vacuum-size",
    "rebuild-ld-cache",
    "systemd-reload",
    "fstrim-all",
    "rebuild-locate-db",
    "regenerate-grub",
];

pub fn validate_action(action: &str) -> Result<(), String> {
    if VALID_ACTIONS.contains(&action) {
        Ok(())
    } else {
        Err(format!("action système inconnue : {action}"))
    }
}

/// Runs one of the 7 fixed privileged system-tool actions, escalating
/// through polkit. Mirrors `packages::install::install_package`'s
/// dedicated-exec-path reasoning: `nitrux-pkexec-system-tools` is its own
/// on-disk copy of the shared wrapper script, never reusing another
/// action's exec path, since pkexec resolves the action purely by exec
/// path with no visibility into argv.
#[tauri::command]
pub fn run_system_tool(action: String) -> Result<String, String> {
    validate_action(&action)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-system-tools", "system-tool", &action],
        Duration::from_secs(60),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_known_action() {
        for action in VALID_ACTIONS {
            assert!(validate_action(action).is_ok(), "should accept {action}");
        }
    }

    #[test]
    fn rejects_unknown_action() {
        assert!(validate_action("rm-rf-root").is_err());
    }

    #[test]
    fn rejects_empty_action() {
        assert!(validate_action("").is_err());
    }

    #[test]
    fn run_system_tool_rejects_unknown_action_before_ever_shelling_out() {
        let result = run_system_tool("bogus-action; rm -rf /".to_string());
        assert!(result.is_err());
    }
}
