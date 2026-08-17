//! Scans recent journald entries for kernel panics, OOM-killer events, and
//! userspace segfaults -- the Linux equivalents of what NiTriTe's BSOD
//! analyzer surfaces on Windows. LogsPage.vue already shows a raw,
//! filterable stream of the last 200 entries, but a crash can easily be
//! buried in there with nothing to distinguish it from routine noise; this
//! scans a much wider window and classifies only the events that actually
//! indicate something crashed.

use crate::logs::{run_journalctl, LogEntry};
use serde::Serialize;

/// Journalctl's default window (LogsPage.vue) is 200 lines -- a crash from
/// hours ago is routinely pushed out of that by routine chatter. 5000 is
/// still fast for journalctl to serve and covers a realistic "what
/// happened since I last rebooted" scan.
const SCAN_WINDOW: u32 = 5000;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub enum CrashKind {
    KernelPanic,
    OomKill,
    Segfault,
    KernelError,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct CrashEvent {
    pub kind: CrashKind,
    pub message: String,
    pub unit: String,
}

/// Classifies a single log entry as a crash event, or `None` if it's
/// routine. Message patterns are the kernel's own well-documented, stable
/// wording (`mm/oom_kill.c`'s "Out of memory: Killed process", the panic
/// path's "Kernel panic - not syncing:", `arch/*/mm/fault.c`'s "segfault
/// at") -- these have been stable across kernel versions for over a decade,
/// unlike free-form CLI tool output that can vary by version/locale.
pub fn classify_crash(entry: &LogEntry) -> Option<CrashEvent> {
    let msg = entry.message.to_lowercase();
    let kind = if msg.contains("kernel panic") {
        CrashKind::KernelPanic
    } else if msg.contains("out of memory") && msg.contains("killed process") {
        CrashKind::OomKill
    } else if msg.contains("segfault at") {
        CrashKind::Segfault
    } else if entry.unit == "kernel" && entry.priority <= 2 {
        // 2 (crit) and below (1=alert, 0=emerg) -- not <=3 (err), which
        // would catch routine-but-noisy driver errors (e.g. USB
        // disconnects) that aren't actually crashes.
        CrashKind::KernelError
    } else {
        return None;
    };
    Some(CrashEvent { kind, message: entry.message.clone(), unit: entry.unit.clone() })
}

#[tauri::command]
pub fn get_crash_events() -> Result<Vec<CrashEvent>, String> {
    let entries = run_journalctl(SCAN_WINDOW)?;
    Ok(entries.iter().filter_map(classify_crash).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(priority: u8, message: &str, unit: &str) -> LogEntry {
        LogEntry { priority, message: message.to_string(), unit: unit.to_string() }
    }

    #[test]
    fn classifies_a_kernel_panic() {
        let e = entry(0, "Kernel panic - not syncing: Fatal exception in interrupt", "kernel");
        let event = classify_crash(&e).expect("should classify as a crash");
        assert_eq!(event.kind, CrashKind::KernelPanic);
    }

    #[test]
    fn classifies_an_oom_kill_with_the_kernels_real_message_format() {
        // Real dmesg/kernel-log wording from mm/oom_kill.c.
        let e = entry(1, "Out of memory: Killed process 4821 (chromium) total-vm:8123456kB, anon-rss:2048576kB", "kernel");
        let event = classify_crash(&e).expect("should classify as a crash");
        assert_eq!(event.kind, CrashKind::OomKill);
    }

    #[test]
    fn classifies_a_userspace_segfault() {
        let e = entry(3, "firefox[12345]: segfault at 0 ip 00007f8a1c2b3d40 sp 00007ffd1a2b3c00 error 4 in libxul.so", "kernel");
        let event = classify_crash(&e).expect("should classify as a crash");
        assert_eq!(event.kind, CrashKind::Segfault);
    }

    #[test]
    fn classifies_a_generic_critical_kernel_message_without_a_matched_keyword() {
        let e = entry(2, "hardware error detected on CPU 0", "kernel");
        let event = classify_crash(&e).expect("should classify as a crash");
        assert_eq!(event.kind, CrashKind::KernelError);
    }

    #[test]
    fn does_not_flag_a_routine_kernel_error_that_is_not_actually_a_crash() {
        // priority 3 (err) is common for noisy-but-harmless driver
        // messages (e.g. a USB device disconnect) -- must not be
        // misclassified as a crash just because it's "kernel" + low-ish
        // priority.
        let e = entry(3, "usb 1-1: device descriptor read/64, error -71", "kernel");
        assert!(classify_crash(&e).is_none());
    }

    #[test]
    fn does_not_flag_a_non_kernel_unit_even_at_high_severity() {
        let e = entry(0, "database connection pool exhausted", "postgresql");
        assert!(classify_crash(&e).is_none());
    }

    #[test]
    fn does_not_flag_routine_informational_log_lines() {
        let e = entry(6, "Starting Daily apt download activities...", "systemd");
        assert!(classify_crash(&e).is_none());
    }

    #[test]
    fn get_crash_events_filters_a_mixed_batch_down_to_only_real_crashes() {
        // Not calling the real #[tauri::command] (needs a live journalctl),
        // but classify_crash is the entire filtering logic it delegates
        // to -- exercised directly against a realistic mixed batch here.
        let entries = [
            entry(6, "Starting Daily apt download activities...", "systemd"),
            entry(1, "Out of memory: Killed process 999 (java) total-vm:4096000kB", "kernel"),
            entry(4, "warning: low disk space", "systemd"),
            entry(0, "Kernel panic - not syncing: Attempted to kill init!", "kernel"),
        ];
        let events: Vec<CrashEvent> = entries.iter().filter_map(classify_crash).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, CrashKind::OomKill);
        assert_eq!(events[1].kind, CrashKind::KernelPanic);
    }
}
