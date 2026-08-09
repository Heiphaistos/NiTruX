import { describe, it, expect } from "vitest";
import { buildPerfHistoryCsv } from "./perfHistoryCsv";

describe("buildPerfHistoryCsv", () => {
  it("produces a header row followed by one row per sample", () => {
    const csv = buildPerfHistoryCsv([
      { timestamp: 1700000000000, cpuPercent: 12.34, memoryPercent: 56.78 },
      { timestamp: 1700000001000, cpuPercent: 99.9, memoryPercent: 0 },
    ]);
    const lines = csv.split("\n");
    expect(lines).toHaveLength(3);
    expect(lines[0]).toBe("Horodatage,CPU (%),Memoire (%)");
  });

  it("rounds percentages to 1 decimal place, not the raw float", () => {
    const csv = buildPerfHistoryCsv([{ timestamp: 1700000000000, cpuPercent: 12.345, memoryPercent: 7 }]);
    expect(csv).toContain("12.3,7.0");
  });

  it("writes the timestamp as an ISO 8601 string, not a raw epoch number", () => {
    const csv = buildPerfHistoryCsv([{ timestamp: 1700000000000, cpuPercent: 0, memoryPercent: 0 }]);
    expect(csv).toContain(new Date(1700000000000).toISOString());
  });

  it("returns just the header when there are no samples yet, not a blank/broken file", () => {
    const csv = buildPerfHistoryCsv([]);
    expect(csv).toBe("Horodatage,CPU (%),Memoire (%)");
  });
});
