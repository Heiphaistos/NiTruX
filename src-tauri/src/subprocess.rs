//! Shared helper for shelling out to external system commands (`lspci`,
//! `lsmod`, `journalctl`, ...) with a bounded wait, so a hung or missing
//! binary can never freeze a Tauri command indefinitely.

use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Runs `program` with `args`, bounding the wait to `timeout`.
///
/// - `Ok(stdout)` — the process exited with status 0; stdout is returned
///   (lossily decoded, since system command output is not guaranteed UTF-8).
/// - `Err(_)` — the binary could not be spawned (e.g. not installed), it
///   exited non-zero, or it did not finish within `timeout`. In the timeout
///   case the child process is sent `SIGKILL` before returning.
pub fn run_with_timeout(program: &str, args: &[&str], timeout: Duration) -> Result<String, String> {
    let child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program} introuvable ou impossible à lancer : {e}"))?;

    let pid = child.id();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        // `wait_with_output` drains stdout/stderr concurrently internally,
        // so (unlike sequential `read_to_string` calls) it cannot deadlock
        // if the child fills one pipe's OS buffer before the other.
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                let code = output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "inconnu".to_string());
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("{program} a échoué (code {code}) : {}", stderr.trim()))
            }
        }
        Ok(Err(e)) => Err(format!("erreur en lisant la sortie de {program} : {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // Best-effort kill; the reader thread above simply exits once
            // the child dies, there is nothing left to join.
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            Err(format!(
                "{program} a dépassé le délai de {timeout:?} et a été arrêté"
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(format!(
            "le processus {program} s'est terminé de façon inattendue"
        )),
    }
}

/// Like `run_with_timeout`, but sets additional environment variables on
/// the child process before spawning it. Needed for commands whose output
/// format is locale-dependent (confirmed during R11's research: `lscpu`
/// genuinely emits translated field labels like "Nom de modèle :" on a
/// French-locale system, which would silently break a parser written
/// against the stable English keys) -- `LC_ALL=C` pins the output to a
/// locale-independent format without affecting any other running process.
pub fn run_with_timeout_env(
    program: &str,
    args: &[&str],
    envs: &[(&str, &str)],
    timeout: Duration,
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    let child = command
        .spawn()
        .map_err(|e| format!("{program} introuvable ou impossible à lancer : {e}"))?;

    let pid = child.id();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                let code = output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "inconnu".to_string());
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("{program} a échoué (code {code}) : {}", stderr.trim()))
            }
        }
        Ok(Err(e)) => Err(format!("erreur en lisant la sortie de {program} : {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            Err(format!(
                "{program} a dépassé le délai de {timeout:?} et a été arrêté"
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(format!(
            "le processus {program} s'est terminé de façon inattendue"
        )),
    }
}

/// Like `run_with_timeout`, but returns stdout regardless of exit code
/// alongside the exit code itself, for callers whose subprocess has a
/// convention where specific non-zero exit codes are meaningful data states
/// rather than failures (e.g. clamscan: 1 = infections found, dnf: 100 =
/// updates available), not just "success" or "hard error". `run_with_timeout`
/// discards stdout whenever the exit code is non-zero, which is correct for
/// commands where non-zero always means failure, but loses data for this
/// class of tool — this helper exists so callers can inspect the code
/// themselves and decide what it means.
///
/// - `Ok((stdout, stderr, code))` — the process ran to completion, whatever
///   its exit code; stdout and stderr are both returned (lossily decoded).
///   The caller decides which codes are success-like and which are real
///   errors, and which stream (if either) carries the real explanation for
///   a given tool — confirmed live (cycles 107-110) that this varies per
///   tool: timeshift/smartctl put their real error text on stdout, tar puts
///   it on stderr (reproduced: a permission-denied file inside the backed-up
///   directory exits 2 with the actionable message ONLY on stderr, stdout
///   empty). Stderr used to be piped (to avoid the child blocking on a full
///   pipe if it wrote enough there) but then silently discarded — every
///   caller was structurally unable to recover a stderr-only error message,
///   regardless of how its own error-formatting code was written.
/// - `Err(_)` — the binary could not be spawned (e.g. not installed), or it
///   did not finish within `timeout` (the child is sent `SIGKILL` first).
pub fn run_capturing_exit_code(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(String, String, i32), String> {
    let child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program} introuvable ou impossible à lancer : {e}"))?;

    let pid = child.id();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            // A negative/absent code means the process was killed by a
            // signal rather than exiting normally; -1 is a safe sentinel
            // since real exit codes are always non-negative.
            let code = output.status.code().unwrap_or(-1);
            Ok((
                String::from_utf8_lossy(&output.stdout).into_owned(),
                String::from_utf8_lossy(&output.stderr).into_owned(),
                code,
            ))
        }
        Ok(Err(e)) => Err(format!("erreur en lisant la sortie de {program} : {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            Err(format!(
                "{program} a dépassé le délai de {timeout:?} et a été arrêté"
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(format!(
            "le processus {program} s'est terminé de façon inattendue"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn returns_ok_stdout_on_success() {
        let out = run_with_timeout("echo", &["hello"], Duration::from_secs(2)).expect("should succeed");
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn returns_err_on_nonzero_exit() {
        let err = run_with_timeout("false", &[], Duration::from_secs(2)).expect_err("should fail");
        assert!(err.contains("false"), "error should mention the program: {err}");
    }

    #[test]
    fn returns_err_when_binary_missing() {
        let err = run_with_timeout("definitely-not-a-real-binary-xyz", &[], Duration::from_secs(2))
            .expect_err("should fail");
        assert!(err.contains("introuvable"));
    }

    #[test]
    fn kills_and_errors_on_timeout() {
        let start = Instant::now();
        let err = run_with_timeout("sleep", &["5"], Duration::from_millis(150)).expect_err("should time out");
        assert!(err.contains("délai"), "error should mention the timeout: {err}");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "should return promptly after killing the child, not wait for it to finish"
        );
    }

    #[test]
    fn run_with_timeout_env_makes_the_env_var_visible_to_the_child() {
        let out = run_with_timeout_env("sh", &["-c", "echo $NITRUX_TEST_VAR"], &[("NITRUX_TEST_VAR", "hello")], Duration::from_secs(2))
            .expect("should succeed");
        assert_eq!(out.trim(), "hello");
    }

    #[test]
    fn run_capturing_exit_code_returns_stdout_and_code_on_zero_exit() {
        let (stdout, _stderr, code) =
            run_capturing_exit_code("sh", &["-c", "echo hello; exit 0"], Duration::from_secs(2))
                .expect("should succeed");
        assert_eq!(stdout.trim(), "hello");
        assert_eq!(code, 0);
    }

    #[test]
    fn run_capturing_exit_code_captures_stdout_on_nonzero_exit_instead_of_erroring() {
        let (stdout, _stderr, code) =
            run_capturing_exit_code("sh", &["-c", "echo hello; exit 1"], Duration::from_secs(2))
                .expect("non-zero exit should still be Ok — caller decides what the code means");
        assert_eq!(stdout.trim(), "hello");
        assert_eq!(code, 1);
    }

    #[test]
    fn run_capturing_exit_code_also_captures_stderr_separately_from_stdout() {
        let (stdout, stderr, code) = run_capturing_exit_code(
            "sh",
            &["-c", "echo on_stdout; echo on_stderr >&2; exit 3"],
            Duration::from_secs(2),
        )
        .expect("non-zero exit should still be Ok");
        assert_eq!(stdout.trim(), "on_stdout");
        assert_eq!(
            stderr.trim(),
            "on_stderr",
            "stderr must be captured, not silently discarded like before this fix"
        );
        assert_eq!(code, 3);
    }

    #[test]
    fn run_capturing_exit_code_returns_err_when_binary_missing() {
        let err = run_capturing_exit_code(
            "definitely-not-a-real-binary-xyz",
            &[],
            Duration::from_secs(2),
        )
        .expect_err("should fail");
        assert!(err.contains("introuvable"));
    }

    #[test]
    fn run_capturing_exit_code_kills_and_errors_on_timeout() {
        let start = Instant::now();
        let err = run_capturing_exit_code("sleep", &["5"], Duration::from_millis(150))
            .expect_err("should time out");
        assert!(err.contains("délai"), "error should mention the timeout: {err}");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "should return promptly after killing the child, not wait for it to finish"
        );
    }
}
