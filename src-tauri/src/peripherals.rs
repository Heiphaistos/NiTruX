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

#[tauri::command]
pub fn get_monitors() -> Result<Vec<String>, String> {
    let out = subprocess::run_with_timeout("xrandr", &["--query"], Duration::from_secs(5))?;
    Ok(out
        .lines()
        .filter(|l| l.contains(" connected"))
        .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
        .collect())
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
    let out = subprocess::run_with_timeout("lpstat", &["-p"], Duration::from_secs(5))
        .map_err(|e| explain_service_error("lpstat", e))?;
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
