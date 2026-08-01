// src/data/appCatalog.spec.ts
import { describe, it, expect } from "vitest";
import { appCatalog } from "./appCatalog";

describe("appCatalog", () => {
  it("has no duplicate ids", () => {
    const ids = appCatalog.map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every entry has non-empty required fields and a valid installMethod", () => {
    const validMethods = ["apt", "flatpak", "snap"];
    for (const entry of appCatalog) {
      expect(entry.id.length).toBeGreaterThan(0);
      expect(entry.name.length).toBeGreaterThan(0);
      expect(entry.description.length).toBeGreaterThan(0);
      expect(entry.icon.length).toBeGreaterThan(0);
      expect(entry.category.length).toBeGreaterThan(0);
      expect(entry.packageId.length).toBeGreaterThan(0);
      expect(validMethods).toContain(entry.installMethod);
    }
  });

  it("contains at least one apt entry and at least one flatpak or snap entry (needed to exercise both the enabled and disabled install UI states)", () => {
    expect(appCatalog.some((e) => e.installMethod === "apt")).toBe(true);
    expect(appCatalog.some((e) => e.installMethod === "flatpak" || e.installMethod === "snap")).toBe(true);
  });
});
