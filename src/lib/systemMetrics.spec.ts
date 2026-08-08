import { describe, it, expect } from "vitest";
import { averageCpuPercent, memoryUsedPercent } from "./systemMetrics";

describe("averageCpuPercent", () => {
  it("averages usage across all cores", () => {
    expect(averageCpuPercent([{ usage_percent: 10 }, { usage_percent: 30 }])).toBe(20);
  });

  it("returns 0 for an empty core list rather than dividing by zero", () => {
    expect(averageCpuPercent([])).toBe(0);
  });
});

describe("memoryUsedPercent", () => {
  it("computes the used/total percentage", () => {
    expect(memoryUsedPercent(4_000_000_000, 8_000_000_000)).toBe(50);
  });

  it("returns 0 when total is 0 rather than dividing by zero", () => {
    expect(memoryUsedPercent(0, 0)).toBe(0);
  });
});
