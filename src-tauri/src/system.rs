use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage_percent: f32,
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

pub fn build_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys
        .cpus()
        .iter()
        .map(|cpu| CpuInfo {
            name: cpu.brand().to_string(),
            usage_percent: cpu.cpu_usage(),
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
pub fn get_system_snapshot() -> SystemSnapshot {
    build_snapshot()
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
        let snap = build_snapshot();
        assert!(!snap.cpus.is_empty());
        assert!(snap.memory_total_bytes > 0);
    }
}
