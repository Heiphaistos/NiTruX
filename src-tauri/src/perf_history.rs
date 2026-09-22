//! Persistent CPU/memory history.
//!
//! `PerfHistoryPage` sampled the system every few seconds and kept the
//! samples in a Vue ref, so closing the window threw the whole history
//! away -- "historique de performance" that only covered the current
//! session. Samples are appended here as one JSON object per line, which
//! means an append is a single write with no read-modify-write of the whole
//! file, and a truncated last line (power loss mid-write) costs exactly one
//! sample instead of the file.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;

/// Roughly 24h at one sample every 30 seconds. The file is rewritten only
/// when it exceeds this, so the common path stays append-only.
const MAX_SAMPLES: usize = 2880;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PerfSample {
    /// Milliseconds since the Unix epoch, matching `Date.now()` on the
    /// frontend side so the existing CSV export needs no conversion.
    pub timestamp_ms: u64,
    pub cpu_percent: f64,
    pub memory_percent: f64,
}

fn history_path() -> Result<PathBuf, String> {
    let base = match std::env::var("XDG_DATA_HOME") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
            PathBuf::from(home).join(".local").join("share")
        }
    };
    let dir = base.join("nitrux");
    std::fs::create_dir_all(&dir).map_err(|e| format!("création de {} impossible : {e}", dir.display()))?;
    Ok(dir.join("perf-history.jsonl"))
}

/// Parses a whole file, skipping any line that does not parse -- a
/// half-written final line must not make the entire history unreadable.
pub fn parse_history(contents: &str) -> Vec<PerfSample> {
    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<PerfSample>(line).ok())
        .collect()
}

pub fn serialize_history(samples: &[PerfSample]) -> String {
    samples
        .iter()
        .filter_map(|s| serde_json::to_string(s).ok())
        .map(|line| format!("{line}\n"))
        .collect()
}

#[tauri::command]
pub fn get_perf_history() -> Result<Vec<PerfSample>, String> {
    let path = history_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(parse_history(&contents)),
        // No file yet is an empty history, not an error: this is the normal
        // state on first launch.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("lecture de {} impossible : {e}", path.display())),
    }
}

#[tauri::command]
pub fn record_perf_sample(cpu_percent: f64, memory_percent: f64) -> Result<(), String> {
    let path = history_path()?;
    let sample = PerfSample {
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        cpu_percent,
        memory_percent,
    };
    let line = serde_json::to_string(&sample).map_err(|e| format!("sérialisation impossible : {e}"))?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("écriture dans {} impossible : {e}", path.display()))?;
    writeln!(file, "{line}").map_err(|e| format!("écriture dans {} impossible : {e}", path.display()))?;
    drop(file);

    trim_if_needed(&path)
}

/// Keeps the newest `MAX_SAMPLES` lines. Cheap check first: the file is
/// only read and rewritten once it actually overflows.
fn trim_if_needed(path: &PathBuf) -> Result<(), String> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };
    let samples = parse_history(&contents);
    if samples.len() <= MAX_SAMPLES {
        return Ok(());
    }
    let kept = &samples[samples.len() - MAX_SAMPLES..];
    std::fs::write(path, serialize_history(kept))
        .map_err(|e| format!("rotation de {} impossible : {e}", path.display()))
}

#[tauri::command]
pub fn clear_perf_history() -> Result<(), String> {
    let path = history_path()?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("suppression de {} impossible : {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_samples_through_the_file_format() {
        let samples = vec![
            PerfSample { timestamp_ms: 1_726_900_000_000, cpu_percent: 12.5, memory_percent: 48.0 },
            PerfSample { timestamp_ms: 1_726_900_030_000, cpu_percent: 99.9, memory_percent: 51.25 },
        ];
        assert_eq!(parse_history(&serialize_history(&samples)), samples);
    }

    #[test]
    fn a_truncated_last_line_costs_one_sample_not_the_whole_history() {
        // Exactly what a power loss mid-append leaves behind.
        let contents = "{\"timestamp_ms\":1,\"cpu_percent\":10.0,\"memory_percent\":20.0}\n{\"timestamp_ms\":2,\"cpu";
        let parsed = parse_history(contents);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].timestamp_ms, 1);
    }

    #[test]
    fn writes_reads_and_rotates_a_real_file() {
        // Points HOME at a scratch directory so this exercises the real
        // path-building, appending and trimming code rather than a stub.
        let scratch = std::env::temp_dir().join(format!("nitrux-perf-test-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        let previous_home = std::env::var("HOME").ok();
        let previous_xdg = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", &scratch);

        clear_perf_history().unwrap();
        assert!(get_perf_history().unwrap().is_empty(), "a missing file is an empty history");

        record_perf_sample(12.5, 48.0).unwrap();
        record_perf_sample(80.0, 50.0).unwrap();
        let history = get_perf_history().unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].cpu_percent, 80.0);
        assert!(history[0].timestamp_ms > 0);

        // Overflow the cap by hand, then one more append must trim back.
        let path = history_path().unwrap();
        let filler: Vec<PerfSample> = (0..MAX_SAMPLES as u64 + 5)
            .map(|i| PerfSample { timestamp_ms: i, cpu_percent: 1.0, memory_percent: 1.0 })
            .collect();
        std::fs::write(&path, serialize_history(&filler)).unwrap();
        record_perf_sample(5.0, 5.0).unwrap();
        let trimmed = get_perf_history().unwrap();
        assert_eq!(trimmed.len(), MAX_SAMPLES);
        assert_eq!(trimmed.last().unwrap().cpu_percent, 5.0, "the newest sample must survive the trim");

        clear_perf_history().unwrap();
        std::fs::remove_dir_all(&scratch).ok();
        match previous_xdg {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        if let Some(home) = previous_home {
            std::env::set_var("HOME", home);
        }
    }
}
