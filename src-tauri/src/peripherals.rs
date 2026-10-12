use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct AudioSink { pub name: String, pub driver: String, pub state: String }

#[derive(Serialize, Clone)]
pub struct PrinterInfo { pub name: String, pub status: String }

/// Parses one `pactl list short sinks` line, e.g.
/// "35    auto_null    PipeWire    float32le 2ch 48000Hz    SUSPENDED" (tab-separated:
/// index, name, driver, format, state).
pub fn parse_pactl_sink_line(line: &str) -> Option<AudioSink> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 5 {
        return None;
    }
    Some(AudioSink {
        name: fields[1].to_string(),
        driver: fields[2].to_string(),
        state: fields[4].to_string(),
    })
}

/// Parses one `lpstat -p` line, e.g. "printer HP_LaserJet is idle." or
/// "printer HP_LaserJet disabled since ...". Only the name and the leading
/// status word after it are extracted -- the rest of CUPS's freeform
/// sentence is not machine-parsed further, it's not needed for a simple
/// name+status display.
pub fn parse_lpstat_line(line: &str) -> Option<PrinterInfo> {
    let rest = line.strip_prefix("printer ")?;
    let (name, status_rest) = rest.split_once(' ')?;
    let status = status_rest.trim_start_matches("is ").split('.').next().unwrap_or("").trim().to_string();
    Some(PrinterInfo { name: name.to_string(), status })
}

// These four commands used to swallow every failure with
// `.unwrap_or_default()` and return an empty list. Their signatures were
// plain `Vec<T>`, so the Tauri IPC layer could never reject -- the
// `try/catch` around each `invoke()` in `PeripheralsPage.vue` was dead code
// for the case that actually happens in the field, and "xrandr is not
// installed" rendered exactly like "this machine has no monitor". Returning
// `Result` is what makes the absence visible at all; the shared package
// hint in `subprocess` then names the package to install.

/// Connected outputs from the kernel's DRM connectors ("DP-3 — E2212F —
/// 1920x1080"). `xrandr` only sees XWayland's virtual outputs on a Wayland
/// session (the default on GNOME, KDE and Hyprland), and needs an extra
/// package; the kernel's view works on X11 and Wayland alike, with nothing
/// to install. `live` adds what only the compositor knows (current refresh
/// rate, scale), keyed by connector name.
fn monitors_from_drm(root: &std::path::Path, live: &std::collections::HashMap<String, String>) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(root).map_err(|e| format!("lecture de {} impossible : {e}", root.display()))?;
    let mut monitors: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let dir = entry.path();
            let file = entry.file_name().to_string_lossy().into_owned();
            // Connectors are "card1-DP-3"; "card1" itself and "renderD128" are not.
            let (_, connector) = file.split_once('-')?;
            let status = std::fs::read_to_string(dir.join("status")).ok()?;
            if status.trim() != "connected" {
                return None;
            }
            let mut parts = vec![connector.to_string()];
            if let Some(name) = std::fs::read(dir.join("edid")).ok().and_then(|e| edid_monitor_name(&e)) {
                parts.push(name);
            }
            match live.get(connector) {
                Some(details) => parts.push(details.clone()),
                None => {
                    let modes = std::fs::read_to_string(dir.join("modes")).unwrap_or_default();
                    if let Some(mode) = modes.lines().next() {
                        parts.push(mode.to_string());
                    }
                }
            }
            Some(parts.join(" — "))
        })
        .collect();
    monitors.sort();
    Ok(monitors)
}

/// The monitor's own name from its EDID ("DELL U2419H"): descriptor blocks
/// at bytes 54/72/90/108, the name one tagged 0xFC, text ending at 0x0A.
fn edid_monitor_name(edid: &[u8]) -> Option<String> {
    [54usize, 72, 90, 108].iter().find_map(|&at| {
        let block = edid.get(at..at + 18)?;
        if block[..3] != [0, 0, 0] || block[3] != 0xFC {
            return None;
        }
        let text: String = block[5..].iter().take_while(|&&b| b != 0x0A).map(|&b| b as char).collect();
        let text = text.trim().to_string();
        (!text.is_empty()).then_some(text)
    })
}

/// Hyprland's live view (`hyprctl monitors -j`): current mode, scale, and
/// the best refresh rate the screen offers when it runs below it.
fn hyprland_details(json: &str) -> std::collections::HashMap<String, String> {
    let Ok(serde_json::Value::Array(monitors)) = serde_json::from_str::<serde_json::Value>(json) else {
        return Default::default();
    };
    monitors
        .iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let (w, h) = (m.get("width")?.as_u64()?, m.get("height")?.as_u64()?);
            let hz = m.get("refreshRate")?.as_f64()?;
            let scale = m.get("scale").and_then(|s| s.as_f64()).unwrap_or(1.0);
            let mut text = format!("{w}x{h} @ {hz:.0} Hz, échelle {scale}");
            let best = m
                .get("availableModes")
                .and_then(|a| a.as_array())
                .into_iter()
                .flatten()
                .filter_map(|mode| mode.as_str()?.strip_prefix(&format!("{w}x{h}@"))?.strip_suffix("Hz")?.parse::<f64>().ok())
                .fold(0.0_f64, f64::max);
            if best > hz + 1.0 {
                text.push_str(&format!(" (l'écran accepte {best:.0} Hz dans cette résolution)"));
            }
            Some((name, text))
        })
        .collect()
}

#[tauri::command]
pub fn get_monitors() -> Result<Vec<String>, String> {
    let live = if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() && subprocess::binary_in_path("hyprctl") {
        subprocess::run_with_timeout("hyprctl", &["monitors", "-j"], Duration::from_secs(5))
            .map(|out| hyprland_details(&out))
            .unwrap_or_default()
    } else {
        Default::default()
    };
    monitors_from_drm(std::path::Path::new("/sys/class/drm"), &live)
}

#[tauri::command]
pub fn get_usb_devices() -> Result<Vec<String>, String> {
    match subprocess::run_with_timeout("lsusb", &[], Duration::from_secs(5)) {
        Ok(out) => Ok(out.lines().map(|l| l.to_string()).collect()),
        // `lsusb` exits 1 with no message at all when the machine has no
        // USB bus to enumerate (VMs without a USB controller, containers),
        // which surfaced as the meaningless "lsusb a échoué (code 1) :".
        // No bus means no USB device: an empty list, not an error. A
        // missing `lsusb` binary still reports its package hint.
        Err(_) if subprocess::binary_in_path("lsusb") && !std::path::Path::new("/dev/bus/usb").exists() => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// Rewrites the two "service not running" failures users actually hit into
/// a sentence that says what to do, instead of the tools' raw output
/// ("pa_context_connect() failed: Connection refused", "Scheduler is not
/// running."). Any other error is passed through untouched.
pub fn explain_service_error(program: &str, error: String) -> String {
    let lower = error.to_lowercase();
    match program {
        "pactl" if lower.contains("connection refused") || lower.contains("connection failure") => {
            "aucun serveur audio (PipeWire ou PulseAudio) n'est en cours d'exécution pour cette session".to_string()
        }
        "lpstat" if lower.contains("scheduler is not running") => {
            "le service d'impression CUPS n'est pas démarré (sudo systemctl start cups)".to_string()
        }
        _ => error,
    }
}

#[tauri::command]
pub fn get_audio_sinks() -> Result<Vec<AudioSink>, String> {
    let out = subprocess::run_with_timeout("pactl", &["list", "short", "sinks"], Duration::from_secs(5))
        .map_err(|e| explain_service_error("pactl", e))?;
    Ok(out.lines().filter_map(parse_pactl_sink_line).collect())
}

#[tauri::command]
pub fn get_printers() -> Result<Vec<PrinterInfo>, String> {
    let out = match subprocess::run_with_timeout("lpstat", &["-p"], Duration::from_secs(5)) {
        // CUPS running, no printer configured: exit 1 "No destinations
        // added." -- an empty list, not an error (seen on Omarchy).
        Err(e) if e.contains("No destinations added") => return Ok(Vec::new()),
        result => result.map_err(|e| explain_service_error("lpstat", e))?,
    };
    Ok(out.lines().filter_map(parse_lpstat_line).collect())
}

/// CUPS queue names are restricted by CUPS itself to printable ASCII
/// without space, `/`, `#` or control characters. Enforcing the narrower
/// shape every real queue uses keeps a name coming from the frontend from
/// turning into an extra `lpoptions` argument.
pub fn validate_printer_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 127 {
        return Err("nom d'imprimante invalide".to_string());
    }
    // A leading dash passes the character check below but reaches
    // `lpoptions` as a flag rather than a queue name.
    if name.starts_with('-') {
        return Err(format!("nom d'imprimante invalide : {name}"));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
        return Err(format!("nom d'imprimante invalide : {name}"));
    }
    Ok(())
}

/// Sets the user's default print queue. Unprivileged on purpose:
/// `lpoptions -d` writes `~/.cups/lpoptions`, a per-user preference, so
/// this needs no elevation at all.
#[tauri::command]
pub fn set_default_printer(name: String) -> Result<String, String> {
    validate_printer_name(&name)?;
    subprocess::run_with_timeout("lpoptions", &["-d", name.as_str()], Duration::from_secs(10))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_monitor_name_from_its_edid() {
        let mut edid = vec![0u8; 128];
        edid[90..95].copy_from_slice(&[0, 0, 0, 0xFC, 0]);
        edid[95..108].copy_from_slice(b"E2212F\n      ");
        assert_eq!(edid_monitor_name(&edid), Some("E2212F".to_string()));
        assert_eq!(edid_monitor_name(&[0u8; 128]), None);
        assert_eq!(edid_monitor_name(&[]), None);
    }

    #[test]
    fn hyprland_details_flag_a_screen_running_below_its_best_refresh_rate() {
        // Shape captured from `hyprctl monitors -j` on Omarchy.
        let json = r#"[{"name":"DP-3","width":1920,"height":1080,"refreshRate":60.0,"scale":1,
            "availableModes":["1920x1080@60.00Hz","1920x1080@100.00Hz","1280x720@120.00Hz"]}]"#;
        let d = hyprland_details(json);
        assert_eq!(d["DP-3"], "1920x1080 @ 60 Hz, échelle 1 (l'écran accepte 100 Hz dans cette résolution)");
        assert!(hyprland_details("not json").is_empty());
    }

    #[test]
    fn lists_only_connected_drm_connectors_with_their_preferred_mode() {
        let root = std::env::temp_dir().join(format!("nitrux-drm-{}", std::process::id()));
        for (name, status, modes) in [
            ("card1-DP-3", "connected", "1920x1080\n1280x720\n"),
            ("card1-HDMI-A-1", "disconnected", ""),
            ("card1-eDP-1", "connected", ""),
        ] {
            std::fs::create_dir_all(root.join(name)).unwrap();
            std::fs::write(root.join(name).join("status"), format!("{status}\n")).unwrap();
            std::fs::write(root.join(name).join("modes"), modes).unwrap();
        }
        std::fs::create_dir_all(root.join("card1")).unwrap();
        let monitors = monitors_from_drm(&root, &Default::default()).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(monitors, vec!["DP-3 — 1920x1080".to_string(), "eDP-1".to_string()]);
    }

    #[test]
    fn explains_a_missing_audio_server_instead_of_the_raw_pactl_error() {
        let raw = "pactl a échoué (code 1) : Connection failure: Connection refused".to_string();
        assert!(explain_service_error("pactl", raw).contains("aucun serveur audio"));
    }

    #[test]
    fn explains_a_stopped_cups_scheduler() {
        let raw = "lpstat a échoué (code 1) : lpstat: Scheduler is not running.".to_string();
        assert!(explain_service_error("lpstat", raw).contains("CUPS n'est pas démarré"));
    }

    #[test]
    fn leaves_other_errors_untouched() {
        let raw = "pactl introuvable ou impossible à lancer".to_string();
        assert_eq!(explain_service_error("pactl", raw.clone()), raw);
    }

    #[test]
    fn accepts_a_real_cups_queue_name() {
        assert!(validate_printer_name("HP_LaserJet-1020.local").is_ok());
    }

    #[test]
    fn refuses_a_printer_name_that_could_become_an_argument() {
        for bad in ["", "-d", "HP LaserJet", "a/b", "x;rm -rf /"] {
            assert!(validate_printer_name(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn parses_a_real_pactl_sink_line() {
        let line = "35\tauto_null\tPipeWire\tfloat32le 2ch 48000Hz\tSUSPENDED";
        let sink = parse_pactl_sink_line(line).expect("should parse");
        assert_eq!(sink.name, "auto_null");
        assert_eq!(sink.driver, "PipeWire");
        assert_eq!(sink.state, "SUSPENDED");
    }

    #[test]
    fn ignores_a_malformed_pactl_line() {
        assert!(parse_pactl_sink_line("not enough fields").is_none());
    }

    #[test]
    fn parses_an_idle_printer_line() {
        let printer = parse_lpstat_line("printer HP_LaserJet is idle.  enabled since Mon 01 Aug").expect("should parse");
        assert_eq!(printer.name, "HP_LaserJet");
        assert_eq!(printer.status, "idle");
    }

    #[test]
    fn ignores_a_non_printer_line() {
        assert!(parse_lpstat_line("system default destination: HP_LaserJet").is_none());
    }
}
