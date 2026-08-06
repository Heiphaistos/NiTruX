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

#[tauri::command]
pub fn get_monitors() -> Vec<String> {
    subprocess::run_with_timeout("xrandr", &["--query"], Duration::from_secs(5))
        .map(|out| {
            out.lines()
                .filter(|l| l.contains(" connected"))
                .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_usb_devices() -> Vec<String> {
    subprocess::run_with_timeout("lsusb", &[], Duration::from_secs(5))
        .map(|out| out.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_audio_sinks() -> Vec<AudioSink> {
    subprocess::run_with_timeout("pactl", &["list", "short", "sinks"], Duration::from_secs(5))
        .map(|out| out.lines().filter_map(parse_pactl_sink_line).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_printers() -> Vec<PrinterInfo> {
    subprocess::run_with_timeout("lpstat", &["-p"], Duration::from_secs(5))
        .map(|out| out.lines().filter_map(parse_lpstat_line).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

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
