use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct CpuDetails {
    pub model_name: Option<String>,
    pub sockets: Option<String>,
    pub cores_per_socket: Option<String>,
    pub threads_per_core: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct BoardDetails {
    pub product_name: Option<String>,
    pub sys_vendor: Option<String>,
    pub board_name: Option<String>,
    pub bios_version: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct MemoryDetails {
    pub total_kb: Option<u64>,
    pub available_kb: Option<u64>,
    pub cached_kb: Option<u64>,
    pub swap_total_kb: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct HardwareDetails {
    pub cpu: CpuDetails,
    pub board: BoardDetails,
    pub memory: MemoryDetails,
}

/// Parses `lscpu` output (run with `LC_ALL=C` by the caller so these
/// English keys are stable). Each field is independently optional -- a
/// missing key (e.g. a virtualized CPU without "Socket(s)") must never
/// hide the fields that ARE present.
pub fn parse_lscpu_output(output: &str) -> CpuDetails {
    let mut model_name = None;
    let mut sockets = None;
    let mut cores_per_socket = None;
    let mut threads_per_core = None;
    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        let value = value.trim().to_string();
        match key.trim() {
            "Model name" => model_name = Some(value),
            "Socket(s)" => sockets = Some(value),
            "Core(s) per socket" => cores_per_socket = Some(value),
            "Thread(s) per core" => threads_per_core = Some(value),
            _ => {}
        }
    }
    CpuDetails { model_name, sockets, cores_per_socket, threads_per_core }
}

/// Reads one `/sys/class/dmi/id/*` field -- readable without root on every
/// system tested during this plan's research (confirmed on the dev VM:
/// `product_name` and `bios_version` both readable as a normal user).
/// Missing entirely on some systems/architectures, so `None` on any read
/// failure is a normal, expected outcome, not an error to surface.
fn read_dmi_field(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn get_board_details() -> BoardDetails {
    BoardDetails {
        product_name: read_dmi_field("product_name"),
        sys_vendor: read_dmi_field("sys_vendor"),
        board_name: read_dmi_field("board_name"),
        bios_version: read_dmi_field("bios_version"),
    }
}

/// Parses `/proc/meminfo` content. Values are in kB per the kernel's own
/// convention (the trailing "kB" unit suffix on each line is discarded,
/// only the leading numeric value is kept).
pub fn parse_meminfo(content: &str) -> MemoryDetails {
    let mut total_kb = None;
    let mut available_kb = None;
    let mut cached_kb = None;
    let mut swap_total_kb = None;
    for line in content.lines() {
        let Some((key, rest)) = line.split_once(':') else { continue };
        let value_kb = rest.split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        match key {
            "MemTotal" => total_kb = value_kb,
            "MemAvailable" => available_kb = value_kb,
            "Cached" => cached_kb = value_kb,
            "SwapTotal" => swap_total_kb = value_kb,
            _ => {}
        }
    }
    MemoryDetails { total_kb, available_kb, cached_kb, swap_total_kb }
}

#[tauri::command]
pub fn get_hardware_details() -> HardwareDetails {
    let cpu = subprocess::run_with_timeout_env("lscpu", &[], &[("LC_ALL", "C")], Duration::from_secs(10))
        .map(|out| parse_lscpu_output(&out))
        .unwrap_or(CpuDetails { model_name: None, sockets: None, cores_per_socket: None, threads_per_core: None });
    let board = get_board_details();
    let memory = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .map(|c| parse_meminfo(&c))
        .unwrap_or(MemoryDetails { total_kb: None, available_kb: None, cached_kb: None, swap_total_kb: None });
    HardwareDetails { cpu, board, memory }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lscpu_english_output() {
        let output = "Architecture:                       x86_64\nCPU(s):                              6\nModel name:                          11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz\nSocket(s):                           1\nCore(s) per socket:                  3\nThread(s) per core:                  2\n";
        let cpu = parse_lscpu_output(output);
        assert_eq!(cpu.model_name.as_deref(), Some("11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz"));
        assert_eq!(cpu.sockets.as_deref(), Some("1"));
        assert_eq!(cpu.cores_per_socket.as_deref(), Some("3"));
        assert_eq!(cpu.threads_per_core.as_deref(), Some("2"));
    }

    #[test]
    fn parse_lscpu_output_leaves_missing_fields_none_without_panicking() {
        let cpu = parse_lscpu_output("Architecture: x86_64\n");
        assert_eq!(cpu.model_name, None);
    }

    #[test]
    fn parses_meminfo_content() {
        let content = "MemTotal:       10231988 kB\nMemFree:          921716 kB\nMemAvailable:    6614532 kB\nCached:          5031560 kB\nSwapTotal:        2097148 kB\n";
        let mem = parse_meminfo(content);
        assert_eq!(mem.total_kb, Some(10231988));
        assert_eq!(mem.available_kb, Some(6614532));
        assert_eq!(mem.cached_kb, Some(5031560));
        assert_eq!(mem.swap_total_kb, Some(2097148));
    }

    #[test]
    fn parse_meminfo_ignores_malformed_lines() {
        let mem = parse_meminfo("not a valid line\nMemTotal:       1000 kB\n");
        assert_eq!(mem.total_kb, Some(1000));
    }
}
