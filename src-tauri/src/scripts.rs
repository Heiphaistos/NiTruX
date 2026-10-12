use crate::subprocess;
use std::time::Duration;

/// Runs `content` as a shell script via `sh -c`, with the invoking user's
/// own privileges only -- no elevation, no pkexec. This is not a new
/// privilege boundary: it is conceptually identical to the user opening a
/// terminal and typing the same command themselves. Bounded to 60 seconds
/// -- long enough for a real utility script, short enough that a runaway
/// script can't hang the app indefinitely.
#[tauri::command]
pub fn run_script(content: String) -> Result<String, String> {
    // The output is shown as-is, so keep the user's language rather than the
    // C locale the parsers need; an empty LC_ALL counts as unset.
    let lc_all = std::env::var("LC_ALL").unwrap_or_default();
    subprocess::run_with_timeout_env("sh", &["-c", &content], &[("LC_ALL", &lc_all)], Duration::from_secs(60))
}

/// Same, as root: `pkexec /bin/sh -c <script>`. polkit's own exec action
/// asks for the admin password every time and shows the command it is
/// about to run; nothing is cached. 5 minutes, password included.
#[tauri::command]
pub fn run_script_as_admin(content: String) -> Result<String, String> {
    if content.trim().is_empty() {
        return Err("script vide".to_string());
    }
    let lc_all = std::env::var("LC_ALL").unwrap_or_default();
    // run_with_timeout (not _env) so the action lands in the admin history.
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/env", &format!("LC_ALL={lc_all}"), "/bin/sh", "-c", &content],
        Duration::from_secs(300),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_a_simple_script_and_captures_stdout() {
        let result = run_script("echo hello".to_string()).expect("should succeed");
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn keeps_the_users_own_locale_for_displayed_output() {
        let expected = std::env::var("LC_ALL").unwrap_or_default();
        let result = run_script("echo \"$LC_ALL\"".to_string()).expect("should succeed");
        assert_eq!(result.trim(), expected);
    }

    #[test]
    fn an_empty_admin_script_never_reaches_pkexec() {
        assert!(run_script_as_admin("   ".to_string()).is_err());
    }

    #[test]
    fn returns_err_for_a_script_that_exits_nonzero() {
        assert!(run_script("exit 1".to_string()).is_err());
    }
}
