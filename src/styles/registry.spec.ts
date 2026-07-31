import { describe, it, expect } from "vitest";
import { styleRegistry } from "./registry";

describe("styleRegistry", () => {
  it("lists exactly 12 styles", () => {
    expect(styleRegistry).toHaveLength(12);
  });

  it("has unique ids", () => {
    const ids = styleRegistry.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every style has a non-empty name and description", () => {
    for (const style of styleRegistry) {
      expect(style.name.length).toBeGreaterThan(0);
      expect(style.description.length).toBeGreaterThan(0);
    }
  });
});
