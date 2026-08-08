// Shared by any page that needs a single CPU/RAM usage percentage derived
// from get_system_snapshot's per-core/byte-count data -- extracted here now
// that a second consumer (DashboardPage's health score) needs the exact
// same formulas PerfHistoryPage already had, matching this project's DRY
// convention (extract on the 2nd real use, not before).

export interface CpuInfoLike {
  usage_percent: number;
}

export function averageCpuPercent(cpus: CpuInfoLike[]): number {
  if (cpus.length === 0) return 0;
  return cpus.reduce((sum, c) => sum + c.usage_percent, 0) / cpus.length;
}

export function memoryUsedPercent(usedBytes: number, totalBytes: number): number {
  if (totalBytes === 0) return 0;
  return (usedBytes / totalBytes) * 100;
}
