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
pub struct AutostartEntry {
    pub name: String,
    /// "unit" for a systemd --user unit, "desktop" for a
    /// `~/.config/autostart/*.desktop` file. They are disabled in
    /// completely different ways, so the frontend must not guess.
    pub kind: String,
    pub enabled: bool,
}

/// A `.desktop` autostart file stays in place when disabled; it is marked.
/// Both keys are honoured by the major desktops, and either one being true
/// (respectively false) means "do not start this".
pub fn desktop_entry_is_enabled(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.eq_ignore_ascii_case("hidden=true") {
            return false;
        }
        if line.to_ascii_lowercase() == "x-gnome-autostart-enabled=false" {
            return false;
        }
    }
    true
}

/// Rewrites a `.desktop` file's content so it is enabled or disabled,
/// leaving every other line untouched -- these files carry the command,
/// icon and translated names, and rewriting them wholesale would lose all
/// of it.
pub fn set_desktop_entry_enabled(content: &str, enabled: bool) -> String {
    let mut out: Vec<String> = content
        .lines()
        .filter(|line| {
            let normalized = line.trim().to_ascii_lowercase();
            !normalized.starts_with("hidden=") && !normalized.starts_with("x-gnome-autostart-enabled=")
        })
        .map(|l| l.to_string())
        .collect();
    if !enabled {
        out.push("Hidden=true".to_string());
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}

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
    let mut entries: Vec<AutostartEntry> = subprocess::run_with_timeout(
        "systemctl",
        &["--user", "list-unit-files", "--state=enabled", "--no-pager", "--plain"],
        Duration::from_secs(10),
    )
    .map(|out| {
        out.lines()
            .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
            // Only units this listing reports, and it was filtered to
            // enabled ones.
            .map(|name| AutostartEntry { name, kind: "unit".to_string(), enabled: true })
            .collect()
    })
    .unwrap_or_default();

    if let Ok(home) = std::env::var("HOME") {
        let autostart_dir = std::path::Path::new(&home).join(".config/autostart");
        if let Ok(dir_entries) = std::fs::read_dir(autostart_dir) {
            for entry in dir_entries.flatten() {
                let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else { continue };
                if !name.ends_with(".desktop") {
                    continue;
                }
                let enabled = std::fs::read_to_string(entry.path())
                    .map(|c| desktop_entry_is_enabled(&c))
                    .unwrap_or(true);
                entries.push(AutostartEntry { name, kind: "desktop".to_string(), enabled });
            }
        }
    }
    entries
}

fn validate_autostart_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 255 {
        return Err("nom d'entrée de démarrage invalide".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.starts_with('-') || name.contains("..") {
        return Err(format!("nom d'entrée de démarrage invalide : {name}"));
    }
    Ok(())
}

/// Enables or disables one autostart entry, unprivileged in both cases: a
/// `.desktop` file under `~/.config/autostart` is the user's own file, and
/// `systemctl --user` acts on the user's own session manager.
#[tauri::command]
pub fn set_autostart_entry_enabled(name: String, kind: String, enabled: bool) -> Result<(), String> {
    validate_autostart_name(&name)?;
    match kind.as_str() {
        "desktop" => {
            let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
            let path = std::path::Path::new(&home).join(".config/autostart").join(&name);
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("lecture de {} impossible : {e}", path.display()))?;
            std::fs::write(&path, set_desktop_entry_enabled(&content, enabled))
                .map_err(|e| format!("écriture de {} impossible : {e}", path.display()))
        }
        "unit" => {
            let action = if enabled { "enable" } else { "disable" };
            subprocess::run_with_timeout(
                "systemctl",
                &["--user", action, "--", name.as_str()],
                Duration::from_secs(20),
            )
            .map(|_| ())
        }
        other => Err(format!("type d'entrée de démarrage inconnu : {other}")),
    }
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
    fn reads_the_disabled_markers_both_desktops_use() {
        let base = "[Desktop Entry]\nType=Application\nExec=/usr/bin/foo\nName=Foo\n";
        assert!(desktop_entry_is_enabled(base));
        assert!(!desktop_entry_is_enabled(&format!("{base}Hidden=true\n")));
        assert!(!desktop_entry_is_enabled(&format!("{base}X-GNOME-Autostart-enabled=false\n")));
    }

    #[test]
    fn disabling_keeps_every_other_line_of_the_desktop_file() {
        // These files carry the command, icon and translated names; a
        // rewrite that dropped them would break the entry permanently.
        let original = "[Desktop Entry]\nType=Application\nExec=/usr/bin/foo --flag\nIcon=foo\nName[fr]=Machin\n";
        let disabled = set_desktop_entry_enabled(original, false);
        assert!(disabled.contains("Exec=/usr/bin/foo --flag"));
        assert!(disabled.contains("Name[fr]=Machin"));
        assert!(!desktop_entry_is_enabled(&disabled));

        let re_enabled = set_desktop_entry_enabled(&disabled, true);
        assert!(desktop_entry_is_enabled(&re_enabled));
        assert!(!re_enabled.contains("Hidden"), "re-enabling must remove the marker: {re_enabled}");
        assert!(re_enabled.contains("Icon=foo"));
    }

    #[test]
    fn refuses_an_autostart_name_that_escapes_the_directory() {
        assert!(validate_autostart_name("foo.desktop").is_ok());
        for bad in ["", "../../.bashrc", "sub/foo.desktop", "-x"] {
            assert!(validate_autostart_name(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn refuses_an_unknown_autostart_kind() {
        let err = set_autostart_entry_enabled("foo.desktop".to_string(), "registry".to_string(), false)
            .expect_err("only desktop and unit exist on Linux");
        assert!(err.contains("inconnu"), "{err}");
    }

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
