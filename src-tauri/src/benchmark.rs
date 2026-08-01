use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

#[derive(Serialize, Clone)]
pub struct BenchmarkResult {
    pub cpu_hashes_per_sec: u64,
    pub disk_write_mbps: f64,
    pub disk_read_mbps: f64,
    pub memory_bandwidth_gbps: f64,
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
pub fn run_benchmark() -> Result<BenchmarkResult, String> {
    let cpu_hashes_per_sec = benchmark_cpu(Duration::from_millis(800));
    let memory_bandwidth_gbps = benchmark_memory(Duration::from_millis(500));
    let (disk_write_mbps, disk_read_mbps) = benchmark_disk(50 * 1024 * 1024)?; // 50 MB

    Ok(BenchmarkResult {
        cpu_hashes_per_sec,
        disk_write_mbps,
        disk_read_mbps,
        memory_bandwidth_gbps,
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
}
