import { describe, it, expect } from "vitest";
import { layoutRegistry } from "./registry";

describe("layoutRegistry", () => {
  it("lists exactly 8 layouts", () => {
    expect(layoutRegistry).toHaveLength(8);
  });

  it("has unique ids", () => {
    const ids = layoutRegistry.map((l) => l.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
