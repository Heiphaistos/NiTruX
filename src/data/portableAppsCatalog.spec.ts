import { describe, it, expect } from "vitest";
import { portableAppsCatalog } from "./portableAppsCatalog";

describe("portableAppsCatalog", () => {
  it("has no duplicate ids", () => {
    const ids = portableAppsCatalog.map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("has no duplicate (githubOwner, githubRepo) pairs", () => {
    const pairs = portableAppsCatalog.map((e) => `${e.githubOwner}/${e.githubRepo}`);
    expect(new Set(pairs).size).toBe(pairs.length);
  });

  it("every entry has non-empty required fields", () => {
    for (const entry of portableAppsCatalog) {
      expect(entry.id.length).toBeGreaterThan(0);
      expect(entry.name.length).toBeGreaterThan(0);
      expect(entry.description.length).toBeGreaterThan(0);
      expect(entry.icon.length).toBeGreaterThan(0);
      expect(entry.category.length).toBeGreaterThan(0);
      expect(entry.githubOwner.length).toBeGreaterThan(0);
      expect(entry.githubRepo.length).toBeGreaterThan(0);
    }
  });

  it("is not empty", () => {
    expect(portableAppsCatalog.length).toBeGreaterThan(0);
  });
});
