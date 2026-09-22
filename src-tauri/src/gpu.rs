//! Discrete GPU status.
//!
//! `sensors.rs` reports whatever hwmon exposes, which covers CPU packages
//! and integrated graphics but says nothing about an NVIDIA card: the
//! proprietary driver does not publish temperature or utilisation through
//! hwmon, only through `nvidia-smi`. Without this, the Temperatures page
//! showed a laptop's CPU at 80 °C and nothing at all about the GPU that was
//! actually heating it.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct GpuStatus {
    pub name: String,
    pub temperature_celsius: Option<f32>,
    pub utilization_percent: Option<f32>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct GpuSnapshot {
    /// False on the overwhelming majority of machines (AMD, Intel, or an
    /// NVIDIA card running nouveau): not an error, just no `nvidia-smi`.
    /// The page says "no NVIDIA GPU detected" rather than showing an error
    /// to someone whose hardware is working perfectly.
    pub nvidia_available: bool,
    pub gpus: Vec<GpuStatus>,
}

fn parse_optional<T: std::str::FromStr>(field: &str) -> Option<T> {
    let trimmed = field.trim();
    // `nvidia-smi` prints "[N/A]" for a value the card does not report
    // (common for utilisation on older or virtualised cards).
    if trimmed.is_empty() || trimmed.starts_with('[') {
        return None;
    }
    trimmed.parse::<T>().ok()
}

/// Parses `nvidia-smi --query-gpu=name,temperature.gpu,utilization.gpu,
/// memory.used,memory.total --format=csv,noheader,nounits`, one line per
/// card: `NVIDIA GeForce RTX 3070, 54, 12, 1024, 8192`.
pub fn parse_nvidia_smi_output(output: &str) -> Vec<GpuStatus> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 5 {
                return None;
            }
            let name = fields[0].trim();
            if name.is_empty() {
                return None;
            }
            Some(GpuStatus {
                name: name.to_string(),
                temperature_celsius: parse_optional(fields[1]),
                utilization_percent: parse_optional(fields[2]),
                memory_used_mb: parse_optional(fields[3]),
                memory_total_mb: parse_optional(fields[4]),
            })
        })
        .collect()
}

#[tauri::command]
pub fn get_gpu_snapshot() -> GpuSnapshot {
    let output = subprocess::run_with_timeout(
        "nvidia-smi",
        &[
            "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ],
        Duration::from_secs(10),
    );
    match output {
        Ok(stdout) => GpuSnapshot { nvidia_available: true, gpus: parse_nvidia_smi_output(&stdout) },
        Err(_) => GpuSnapshot { nvidia_available: false, gpus: Vec::new() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_two_card_output() {
        let output = "NVIDIA GeForce RTX 3070, 54, 12, 1024, 8192\nNVIDIA T400, 41, 0, 12, 2048\n";
        let gpus = parse_nvidia_smi_output(output);
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].name, "NVIDIA GeForce RTX 3070");
        assert_eq!(gpus[0].temperature_celsius, Some(54.0));
        assert_eq!(gpus[1].memory_total_mb, Some(2048));
    }

    #[test]
    fn keeps_a_card_whose_utilisation_is_not_reported() {
        // Real output from a virtualised card: the name and temperature are
        // still worth showing, so "[N/A]" must not drop the whole row.
        let gpus = parse_nvidia_smi_output("NVIDIA A100-SXM, 38, [N/A], 0, 40960\n");
        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].utilization_percent, None);
        assert_eq!(gpus[0].temperature_celsius, Some(38.0));
    }

    #[test]
    fn ignores_malformed_lines() {
        assert!(parse_nvidia_smi_output("\n\ngarbage\n").is_empty());
    }

    #[test]
    fn reports_unavailable_rather_than_failing_when_nvidia_smi_is_absent() {
        // True on this dev machine and on every AMD/Intel system: absence
        // is the normal case and must never surface as an error.
        let snapshot = get_gpu_snapshot();
        if !snapshot.nvidia_available {
            assert!(snapshot.gpus.is_empty());
        }
    }
}
