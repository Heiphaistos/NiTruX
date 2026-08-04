use crate::subprocess;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Serialize, Clone)]
pub struct AutostartEntry { pub name: String }

/// Parses one line of `systemctl list-timers --no-pager` output. The
/// header row and the trailing summary line ("N timers listed.") are both
/// naturally rejected by requiring a line to end in ".timer" on its unit
/// column -- simpler and more robust than trying to detect the header by
/// position, since column widths vary with content.
pub fn parse_timer_line(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| token.ends_with(".timer"))
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn get_processes(state: tauri::State<Mutex<System>>) -> Vec<ProcessInfo> {
    let mut sys = state.lock().expect("system state mutex poisoned");
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cpu_percent: process.cpu_usage(),
            memory_bytes: process.memory(),
        })
        .collect()
}

#[tauri::command]
pub fn get_systemd_services() -> Vec<String> {
    subprocess::run_with_timeout("systemctl", &["list-units", "--type=service", "--no-pager", "--plain"], Duration::from_secs(10))
        .map(|out| {
            out.lines()
                .filter(|l| l.contains(".service"))
                .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_autostart_entries() -> Vec<AutostartEntry> {
    let mut names: Vec<String> = subprocess::run_with_timeout(
        "systemctl",
        &["--user", "list-unit-files", "--state=enabled", "--no-pager", "--plain"],
        Duration::from_secs(10),
    )
    .map(|out| out.lines().filter_map(|l| l.split_whitespace().next().map(|s| s.to_string())).collect())
    .unwrap_or_default();

    if let Ok(home) = std::env::var("HOME") {
        let autostart_dir = std::path::Path::new(&home).join(".config/autostart");
        if let Ok(entries) = std::fs::read_dir(autostart_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        names.push(name.to_string());
                    }
                }
            }
        }
    }
    names.into_iter().map(|name| AutostartEntry { name }).collect()
}

#[tauri::command]
pub fn get_scheduled_tasks() -> Vec<String> {
    let mut tasks: Vec<String> = subprocess::run_with_timeout("crontab", &["-l"], Duration::from_secs(5))
        .map(|out| {
            out.lines()
                .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    let timers = subprocess::run_with_timeout("systemctl", &["list-timers", "--no-pager", "--plain"], Duration::from_secs(10))
        .map(|out| out.lines().filter_map(parse_timer_line).collect::<Vec<_>>())
        .unwrap_or_default();
    tasks.extend(timers);
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_systemd_timer_line() {
        let line = "Sun 2026-08-02 02:22:55 CEST    17min Sun 2026-08-02 01:32:38 CEST    32min ago fwupd-refresh.timer          fwupd-refresh.service";
        assert_eq!(parse_timer_line(line), Some("fwupd-refresh.timer".to_string()));
    }

    #[test]
    fn ignores_the_timer_list_header_line() {
        assert_eq!(parse_timer_line("NEXT                             LEFT LAST                               PASSED UNIT                         ACTIVATES"), None);
    }

    #[test]
    fn ignores_the_timer_list_summary_line() {
        assert_eq!(parse_timer_line("4 timers listed."), None);
    }
}
