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
    subprocess::run_with_timeout("sh", &["-c", &content], Duration::from_secs(60))
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
    fn returns_err_for_a_script_that_exits_nonzero() {
        assert!(run_script("exit 1".to_string()).is_err());
    }
}
