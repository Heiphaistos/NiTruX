import { describe, it, expect } from "vitest";
import { navigationCategories } from "./categories";

describe("navigationCategories", () => {
  it("has exactly 7 categories", () => {
    expect(navigationCategories).toHaveLength(7);
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
});
