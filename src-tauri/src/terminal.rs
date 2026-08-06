//! Integrated terminal: opens a real pseudo-terminal running the invoking
//! user's own shell. Deliberately NOT a new privilege boundary -- the
//! spawned shell inherits exactly the permissions of the user who launched
//! NiTruX, identical to opening a normal terminal window. This is the
//! first bidirectional-streaming feature in the app (every other command
//! is a one-shot request/response); output streams continuously to the
//! frontend via a Tauri `Channel` rather than a single returned value.

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::ipc::Channel;

pub struct TerminalSession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
}

type PtyHandles = (Box<dyn MasterPty + Send>, Box<dyn Write + Send>, Box<dyn Read + Send>, Box<dyn Child + Send + Sync>);

#[derive(Default)]
pub struct TerminalState(pub Mutex<HashMap<String, TerminalSession>>);

/// Opens a pty and spawns the user's shell in it. Factored out from the
/// `spawn_terminal` Tauri command so it can be exercised in a real test
/// without needing a live Tauri app context (a `Channel` cannot be
/// constructed outside one) -- this is the actual mechanism under test,
/// the command wrapper below just plumbs its output into a Channel.
fn open_shell_pty(rows: u16, cols: u16) -> Result<PtyHandles, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| format!("impossible d'ouvrir le pseudo-terminal : {e}"))?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let cmd = CommandBuilder::new(shell);
    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("impossible de lancer le shell : {e}"))?;
    // The slave end must be dropped in the parent process after spawning --
    // the child process holds its own copy of the fd; keeping the parent's
    // copy open would prevent ever seeing EOF from the master reader.
    drop(pair.slave);

    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("impossible de lire la sortie du terminal : {e}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("impossible d'écrire vers le terminal : {e}"))?;

    Ok((pair.master, writer, reader, child))
}

/// Inserts `session` under `id`, killing whatever session (and its shell
/// child process) previously lived under that same id instead of letting
/// `HashMap::insert` silently drop and orphan it. Not reachable via the
/// current frontend (each TerminalPage instance generates a fresh UUID
/// once, and KeepAlive means `spawn_terminal` is only ever called once per
/// instance -- see App.spec.ts's dedicated KeepAlive test), but a real,
/// cheap-to-close gap: unlike `close_terminal`, plain `insert` would leak
/// a whole running shell process with no way left to reach or kill it.
/// Same "fix even though not currently exploitable, for consistency with
/// this codebase's established defensive-validation discipline" rationale
/// already applied to `trash.rs`'s path validation.
fn insert_session(sessions: &mut HashMap<String, TerminalSession>, id: String, session: TerminalSession) {
    if let Some(mut previous) = sessions.insert(id, session) {
        let _ = previous.child.kill();
    }
}

#[tauri::command]
pub fn spawn_terminal(
    id: String,
    state: tauri::State<TerminalState>,
    on_data: Channel<String>,
) -> Result<(), String> {
    let (master, writer, mut reader, child) = open_shell_pty(24, 80)?;

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                    if on_data.send(chunk).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    insert_session(&mut state.0.lock().unwrap(), id, TerminalSession { master, writer, child });
    Ok(())
}

fn write_to_session(sessions: &mut HashMap<String, TerminalSession>, id: &str, data: &str) -> Result<(), String> {
    let session = sessions.get_mut(id).ok_or("session de terminal introuvable")?;
    session
        .writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("échec de l'écriture vers le terminal : {e}"))
}

#[tauri::command]
pub fn write_to_terminal(id: String, data: String, state: tauri::State<TerminalState>) -> Result<(), String> {
    let mut sessions = state.0.lock().unwrap();
    write_to_session(&mut sessions, &id, &data)
}

#[tauri::command]
pub fn resize_terminal(id: String, rows: u16, cols: u16, state: tauri::State<TerminalState>) -> Result<(), String> {
    let sessions = state.0.lock().unwrap();
    let session = sessions.get(&id).ok_or("session de terminal introuvable")?;
    session
        .master
        .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| format!("échec du redimensionnement : {e}"))
}

#[tauri::command]
pub fn close_terminal(id: String, state: tauri::State<TerminalState>) -> Result<(), String> {
    let mut sessions = state.0.lock().unwrap();
    if let Some(mut session) = sessions.remove(&id) {
        let _ = session.child.kill();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    /// Spawns a real shell in a real pty (this runs under WSL2/Linux in
    /// this project's test environment, so a genuine `bash` process is
    /// launched), writes a command with a unique marker, and confirms the
    /// marker comes back through the pty's output -- exercising the actual
    /// mechanism the streaming commands above are built on, not a mock.
    /// Reading blocks, so a background thread + mpsc + recv_timeout bounds
    /// the wait, mirroring the established pattern in
    /// `subprocess::run_with_timeout`.
    #[test]
    fn spawns_a_real_shell_and_echoes_a_command() {
        let (_master, mut writer, mut reader, _child) = open_shell_pty(24, 80).expect("should open pty");

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut collected = String::new();
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        collected.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if collected.contains("PTY_TEST_MARKER_12345") {
                            let _ = tx.send(collected);
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(collected);
        });

        writer.write_all(b"echo PTY_TEST_MARKER_12345\n").expect("should write to pty");

        let output = rx.recv_timeout(Duration::from_secs(5)).expect("should receive output before timing out");
        assert!(output.contains("PTY_TEST_MARKER_12345"), "pty output should contain the echoed marker: {output}");
    }

    #[test]
    fn write_to_session_errors_cleanly_for_an_unknown_session() {
        let mut sessions: HashMap<String, TerminalSession> = HashMap::new();
        let result = write_to_session(&mut sessions, "nonexistent-id", "echo hi\n");
        assert!(result.is_err());
    }

    /// `/proc/<pid>` keeps existing for a killed-but-not-yet-reaped process
    /// (zombie state "Z" in the 3rd whitespace-separated field of
    /// /proc/<pid>/stat, per proc(5)) -- so "still really running" means
    /// the path exists AND the state isn't "Z", not just path existence.
    fn proc_is_running(pid: u32) -> bool {
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            return false; // no such pid at all
        };
        // comm (2nd field) is parenthesized and may itself contain spaces,
        // so the state is the first token after the LAST ')', per proc(5).
        let Some(after_comm) = stat.rsplit_once(')').map(|(_, rest)| rest) else {
            return false;
        };
        after_comm.split_whitespace().next() != Some("Z")
    }

    /// Regression guard for the actual gap: plain `HashMap::insert` would
    /// silently drop (and, per portable_pty's Child trait, never
    /// implicitly kill) whatever real shell process previously lived under
    /// a reused id, leaking a running process with no way left to reach
    /// it. Uses two genuinely spawned shells (real ptys, like the pty test
    /// above) rather than a mock -- the first child's real OS pid is
    /// captured before it's moved into the session, then `/proc/<pid>`
    /// (this project's test environment is confirmed Linux/WSL2 by the pty
    /// test above) is checked after the reused-id insert to prove the
    /// actual process was killed, not just that a HashMap entry was
    /// overwritten.
    #[test]
    fn insert_session_kills_the_previous_session_under_a_reused_id() {
        let (master1, writer1, _reader1, child1) = open_shell_pty(24, 80).expect("should open first pty");
        let first_pid = child1.process_id().expect("a freshly spawned shell should have a pid");
        let (master2, writer2, _reader2, child2) = open_shell_pty(24, 80).expect("should open second pty");

        let mut sessions: HashMap<String, TerminalSession> = HashMap::new();
        insert_session(&mut sessions, "reused-id".to_string(), TerminalSession { master: master1, writer: writer1, child: child1 });
        assert!(
            proc_is_running(first_pid),
            "sanity check: the first shell should still be running right after being inserted"
        );

        insert_session(&mut sessions, "reused-id".to_string(), TerminalSession { master: master2, writer: writer2, child: child2 });

        // Exactly one session remains under the id, and it's still usable.
        assert_eq!(sessions.len(), 1);
        write_to_session(&mut sessions, "reused-id", "echo still-alive\n").expect("the second session should still be writable");

        // The first shell's actual OS process must have been killed, not
        // merely dropped from the map -- poll briefly since the kernel may
        // take an instant to transition it to zombie/gone after the kill
        // signal is sent.
        let mut still_running = proc_is_running(first_pid);
        for _ in 0..20 {
            if !still_running {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
            still_running = proc_is_running(first_pid);
        }
        assert!(!still_running, "the first shell's process ({first_pid}) should have been killed when its id was reused, not leaked");
    }
}
