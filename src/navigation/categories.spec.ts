import { describe, it, expect } from "vitest";
import { navigationCategories } from "./categories";

describe("navigationCategories", () => {
  it("has exactly 8 categories", () => {
    expect(navigationCategories).toHaveLength(8);
  });

  it("every page id is unique across all categories", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(new Set(allPageIds).size).toBe(allPageIds.length);
  });

  it("every category has at least one page", () => {
    for (const category of navigationCategories) {
      expect(category.pages.length).toBeGreaterThan(0);
    }
  });

  it("includes the 4 new Phase R1+ feature pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("quick-install");
    expect(allPageIds).toContain("updates");
    expect(allPageIds).toContain("report-generator");
    expect(allPageIds).toContain("settings-preferences");
  });

  it("includes the 4 new Phase R6 Performance category pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("optimizations");
    expect(allPageIds).toContain("temperatures");
    expect(allPageIds).toContain("benchmark");
    expect(allPageIds).toContain("perf-history");
  });

  it("includes the 5 new Phase R7 Maintenance avancée pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("uninstaller");
    expect(allPageIds).toContain("cleaner");
    expect(allPageIds).toContain("backup");
    expect(allPageIds).toContain("antivirus");
    expect(allPageIds).toContain("dependencies");
  });

  it("includes the 4 new Phase R8 Réseau & Terminal pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("dns-switcher");
    expect(allPageIds).toContain("wifi-analyzer");
    expect(allPageIds).toContain("bluetooth");
    expect(allPageIds).toContain("scripts");
  });

  it("includes the 4 new Phase R9 Stockage avancé pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("disk-visualizer");
    expect(allPageIds).toContain("data-recovery");
    expect(allPageIds).toContain("boot-manager");
    expect(allPageIds).toContain("restore-points");
  });

  it("includes the new Phase R10 Logiciels & déploiement page by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("install-profiles");
  });
});
