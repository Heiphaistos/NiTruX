use crate::disks;
use crate::smart;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct DiskHealthEntry {
    pub device: String,
    /// `None` covers both "smartctl absent/inaccessible" and "no health
    /// line in its output" -- same honest, non-distinguishing behavior as
    /// `smart::SmartStatus::health` itself, which this reuses directly.
    pub health: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct BenchmarkResult {
    pub cpu_hashes_per_sec: u64,
    pub disk_write_mbps: f64,
    pub disk_read_mbps: f64,
    pub memory_bandwidth_gbps: f64,
    pub cpu_frequency_mhz: u64,
    pub disk_health: Vec<DiskHealthEntry>,
}

/// Reads the first CPU core's clock speed from the shared, long-lived
/// `System` instance (same instance `system::build_snapshot` refreshes
/// for CPU usage -- see that function's doc comment on why a fresh
/// `System::new_all()` per call is the wrong pattern for usage deltas;
/// frequency itself has no such delta requirement, but reusing the
/// managed instance avoids a second full system scan). `0` if sysinfo
/// cannot determine it on this host, a real and honest value, not an
/// error -- frequency reporting support varies by CPU/kernel.
pub fn read_cpu_frequency_mhz(sys: &mut System) -> u64 {
    sys.refresh_cpu_usage();
    sys.cpus().first().map(sysinfo::Cpu::frequency).unwrap_or(0)
}

/// Best-effort SMART health per detected disk. A disk whose health can't
/// be determined (smartctl absent, permission denied, unsupported
/// device) reports `health: None` rather than failing the whole
/// benchmark -- matches this codebase's established "each source
/// degrades independently" philosophy (network.rs, docker.rs, ...): one
/// unreadable disk must never hide results for the others, or for every
/// other benchmark metric that already ran successfully.
pub fn collect_disk_health(detected_disks: &[disks::Disk]) -> Vec<DiskHealthEntry> {
    detected_disks
        .iter()
        .map(|d| {
            let device = format!("/dev/{}", d.name);
            let health = smart::get_smart_status(device.clone()).ok().and_then(|s| s.health);
            DiskHealthEntry { device, health }
        })
        .collect()
}

/// Hashes a fixed 1 KB buffer repeatedly for `duration`, returning the
/// count of completed SHA-256 operations. The buffer content is irrelevant
/// (this measures raw hashing throughput, not any real data), so it is
/// zero-filled once and reused for every iteration.
pub fn benchmark_cpu(duration: Duration) -> u64 {
    let buf = [0u8; 1024];
    let mut count: u64 = 0;
    let start = Instant::now();
    while start.elapsed() < duration {
        let mut hasher = Sha256::new();
        hasher.update(buf);
        let _ = hasher.finalize();
        count += 1;
    }
    count
}

/// Allocates a fixed-size buffer and repeatedly copies it into a second
/// buffer for `duration`, returning bandwidth in GB/s. `std::hint::black_box`
/// prevents the compiler from optimizing the copy away entirely, since its
/// result is otherwise never observed.
pub fn benchmark_memory(duration: Duration) -> f64 {
    const CHUNK_BYTES: usize = 16 * 1024 * 1024; // 16 MB
    let src = vec![0xABu8; CHUNK_BYTES];
    let mut dst = vec![0u8; CHUNK_BYTES];
    let start = Instant::now();
    let mut bytes_copied: u64 = 0;
    while start.elapsed() < duration {
        dst.copy_from_slice(&src);
        std::hint::black_box(&dst);
        bytes_copied += CHUNK_BYTES as u64;
    }
    let elapsed_secs = start.elapsed().as_secs_f64();
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    (bytes_copied as f64 / elapsed_secs) / 1_073_741_824.0
}

/// Writes then reads back a fixed-size temp file, measuring MB/s for each
/// direction. The file is created under `std::env::temp_dir()` (never a
/// system path) and removed before returning, including on the error path,
/// so a failed read never leaves a stray multi-megabyte file behind.
pub fn benchmark_disk(size_bytes: usize) -> Result<(f64, f64), String> {
    let path = std::env::temp_dir().join(format!("nitrux-benchmark-{}.tmp", std::process::id()));
    let data = vec![0x5Au8; size_bytes];

    let write_result = (|| -> Result<f64, String> {
        let start = Instant::now();
        let mut file = std::fs::File::create(&path).map_err(|e| format!("création du fichier de test impossible : {e}"))?;
        file.write_all(&data).map_err(|e| format!("écriture impossible : {e}"))?;
        file.sync_all().map_err(|e| format!("synchronisation disque impossible : {e}"))?;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return Ok(0.0);
        }
        Ok((size_bytes as f64 / elapsed) / 1_048_576.0)
    })();

    let read_result = write_result.clone().and_then(|_| {
        let start = Instant::now();
        let mut file = std::fs::File::open(&path).map_err(|e| format!("lecture impossible : {e}"))?;
        let mut buf = Vec::with_capacity(size_bytes);
        file.read_to_end(&mut buf).map_err(|e| format!("lecture impossible : {e}"))?;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return Ok(0.0);
        }
        Ok((size_bytes as f64 / elapsed) / 1_048_576.0)
    });

    let _ = std::fs::remove_file(&path);

    let write_mbps = write_result?;
    let read_mbps = read_result?;
    Ok((write_mbps, read_mbps))
}

#[tauri::command]
pub fn run_benchmark(state: tauri::State<Mutex<System>>) -> Result<BenchmarkResult, String> {
    let cpu_hashes_per_sec = benchmark_cpu(Duration::from_millis(800));
    let memory_bandwidth_gbps = benchmark_memory(Duration::from_millis(500));
    let (disk_write_mbps, disk_read_mbps) = benchmark_disk(50 * 1024 * 1024)?; // 50 MB

    let cpu_frequency_mhz = {
        let mut sys = state.lock().expect("system state mutex poisoned");
        read_cpu_frequency_mhz(&mut sys)
    };
    let disk_health = collect_disk_health(&disks::list_disks().unwrap_or_default());

    Ok(BenchmarkResult {
        cpu_hashes_per_sec,
        disk_write_mbps,
        disk_read_mbps,
        memory_bandwidth_gbps,
        cpu_frequency_mhz,
        disk_health,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_cpu_completes_at_least_one_hash_in_a_nonzero_window() {
        let count = benchmark_cpu(Duration::from_millis(50));
        assert!(count > 0, "expected at least one SHA-256 op in 50ms, got {count}");
    }

    #[test]
    fn benchmark_memory_reports_a_positive_bandwidth() {
        let gbps = benchmark_memory(Duration::from_millis(50));
        assert!(gbps > 0.0, "expected positive bandwidth, got {gbps}");
    }

    #[test]
    fn benchmark_disk_writes_reads_and_cleans_up_the_temp_file() {
        let size = 1024 * 1024; // 1 MB, small and fast for a test
        let (write_mbps, read_mbps) = benchmark_disk(size).expect("benchmark should succeed on a normal filesystem");
        assert!(write_mbps > 0.0, "expected positive write throughput, got {write_mbps}");
        assert!(read_mbps > 0.0, "expected positive read throughput, got {read_mbps}");

        let path = std::env::temp_dir().join(format!("nitrux-benchmark-{}.tmp", std::process::id()));
        assert!(!path.exists(), "temp file should be removed after the benchmark runs");
    }

    #[test]
    fn read_cpu_frequency_mhz_returns_a_real_value_on_this_host() {
        // No fabricated expectation on the exact number (hardware-dependent),
        // but on a real host sysinfo must report *some* nonzero frequency
        // for at least the reading not to silently regress into the
        // "always 0" bug this function exists to avoid.
        let mut sys = System::new_all();
        let mhz = read_cpu_frequency_mhz(&mut sys);
        assert!(mhz > 0, "expected a nonzero CPU frequency on a real host, got {mhz}");
    }

    #[test]
    fn collect_disk_health_returns_one_entry_per_disk_with_the_dev_prefixed_device_path() {
        let fixture_disks = vec![
            disks::Disk { name: "sda".to_string(), size: "500G".to_string(), partitions: vec![] },
            disks::Disk { name: "nvme0n1".to_string(), size: "1T".to_string(), partitions: vec![] },
        ];
        let health = collect_disk_health(&fixture_disks);
        assert_eq!(health.len(), 2);
        assert_eq!(health[0].device, "/dev/sda");
        assert_eq!(health[1].device, "/dev/nvme0n1");
    }

    #[test]
    fn collect_disk_health_never_panics_when_smartctl_is_genuinely_unavailable() {
        // smartctl is confirmed absent in this project's WSL2 dev
        // environment (see cycle 53's smart.rs work) -- get_smart_status
        // must degrade to health: None here, not propagate a panic or
        // drop the disk entry entirely.
        let fixture_disks = vec![disks::Disk { name: "sda".to_string(), size: "500G".to_string(), partitions: vec![] }];
        let health = collect_disk_health(&fixture_disks);
        assert_eq!(health.len(), 1);
        assert_eq!(health[0].health, None);
    }

    #[test]
    fn collect_disk_health_returns_an_empty_vec_for_no_disks() {
        assert!(collect_disk_health(&[]).is_empty());
    }
}
