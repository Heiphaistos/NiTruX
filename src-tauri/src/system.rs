use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Serialize, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage_percent: f32,
    pub usage_display: String,
}

#[derive(Serialize, Clone)]
pub struct SystemSnapshot {
    pub cpus: Vec<CpuInfo>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub process_count: usize,
}

pub fn format_percent(value: f32) -> String {
    format!("{:.1}%", value)
}

/// Builds a snapshot from an existing `System` instance.
///
/// CPU usage in sysinfo is computed as a delta between the previous sample
/// and the current one, so `sys` MUST be a long-lived instance that gets
/// refreshed repeatedly (e.g. held in Tauri managed state) rather than a
/// fresh `System::new_all()` constructed on every call — otherwise there is
/// never a prior sample and usage always reads ~0%.
pub fn build_snapshot(sys: &mut System) -> SystemSnapshot {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let cpus = sys
        .cpus()
        .iter()
        .map(|cpu| CpuInfo {
            name: cpu.brand().to_string(),
            usage_percent: cpu.cpu_usage(),
            usage_display: format_percent(cpu.cpu_usage()),
        })
        .collect();

    SystemSnapshot {
        cpus,
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
        process_count: sys.processes().len(),
    }
}

#[tauri::command]
pub fn get_system_snapshot(state: tauri::State<Mutex<System>>) -> SystemSnapshot {
    let mut sys = state.lock().expect("system state mutex poisoned");
    build_snapshot(&mut sys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_percent_rounds_to_one_decimal() {
        assert_eq!(format_percent(62.456), "62.5%");
        assert_eq!(format_percent(0.0), "0.0%");
        assert_eq!(format_percent(100.0), "100.0%");
    }

    #[test]
    fn snapshot_has_at_least_one_cpu_and_nonzero_memory() {
        let mut sys = System::new_all();
        let snap = build_snapshot(&mut sys);
        assert!(!snap.cpus.is_empty());
        assert!(snap.memory_total_bytes > 0);
    }

    #[test]
    fn usage_display_matches_format_percent_of_usage_percent() {
        let mut sys = System::new_all();
        let snap = build_snapshot(&mut sys);
        for cpu in &snap.cpus {
            assert_eq!(cpu.usage_display, format_percent(cpu.usage_percent));
        }
    }

    /// Regression test for the "CPU usage always reads ~0%" bug: sysinfo
    /// computes usage as a delta between refreshes, so refreshing the SAME
    /// `System` instance twice (with real CPU activity and at least
    /// `MINIMUM_CPU_UPDATE_INTERVAL` between refreshes) must be able to
    /// produce a nonzero reading. This is timing-sensitive (depends on OS
    /// scheduling), so it's marked `#[ignore]` to avoid CI flakiness — run
    /// explicitly with `cargo test -- --ignored` to verify the fix.
    #[test]
    #[ignore = "timing-sensitive: run with `cargo test system:: -- --ignored` to verify manually"]
    fn repeated_refresh_on_shared_system_computes_nonzero_cpu_delta() {
        let mut sys = System::new_all();
        // Seed the baseline sample.
        let _first = build_snapshot(&mut sys);

        // Burn CPU on this thread for at least the minimum interval so
        // there is guaranteed measurable usage on at least one core.
        let start = std::time::Instant::now();
        let mut acc: u64 = 0;
        while start.elapsed() < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            acc = acc.wrapping_add(1);
        }
        std::hint::black_box(acc);

        let second = build_snapshot(&mut sys);
        assert!(
            second.cpus.iter().any(|c| c.usage_percent > 0.0),
            "expected at least one CPU to report nonzero usage after a real delta refresh on a shared System instance"
        );
    }
}
