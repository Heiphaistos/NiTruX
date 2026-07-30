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
}
