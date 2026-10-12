import { describe, it, expect } from "vitest";
import { diffReportLines } from "./reportDiff";

describe("diffReportLines", () => {
  it("lists what disappeared and what appeared, ignoring blank lines and order", () => {
    const before = "CPU: 4 cœurs\nRAM libre: 2 Go\n\nNoyau: 6.1\n";
    const after = "Noyau: 6.6\nCPU: 4 cœurs\nRAM libre: 5 Go\n";
    expect(diffReportLines(before, after)).toEqual({
      removed: ["RAM libre: 2 Go", "Noyau: 6.1"],
      added: ["Noyau: 6.6", "RAM libre: 5 Go"],
    });
  });

  it("finds nothing between identical reports", () => {
    expect(diffReportLines("a\nb", "a\nb\n")).toEqual({ removed: [], added: [] });
  });
});
