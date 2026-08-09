// Extracted from PerfHistoryPage.vue so the CSV-building logic can be unit
// tested directly, without mounting the component or mocking Blob/URL/anchor
// click DOM side effects -- same separation-of-concerns already used by
// systemMetrics.ts for this page's other pure calculations.

export interface PerfSample {
  timestamp: number;
  cpuPercent: number;
  memoryPercent: number;
}

export function buildPerfHistoryCsv(samples: PerfSample[]): string {
  const header = "Horodatage,CPU (%),Memoire (%)";
  const rows = samples.map(
    (s) => `${new Date(s.timestamp).toISOString()},${s.cpuPercent.toFixed(1)},${s.memoryPercent.toFixed(1)}`,
  );
  return [header, ...rows].join("\n");
}
