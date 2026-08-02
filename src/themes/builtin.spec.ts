import { describe, it, expect } from "vitest";
import { builtinThemes } from "./builtin";

describe("builtinThemes", () => {
  it("ships exactly 13 themes", () => {
    expect(builtinThemes).toHaveLength(13);
  });

  it("has unique ids", () => {
    const ids = builtinThemes.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every theme defines all required color keys", () => {
    const requiredKeys = [
      "bgBase", "bgElevated", "bgOverlay", "border",
      "textPrimary", "textSecondary",
      "accentPrimary", "accentSecondary", "accentSuccess", "accentWarning", "accentDanger",
    ];
    for (const theme of builtinThemes) {
      for (const key of requiredKeys) {
        expect(theme.colors, `${theme.id} missing ${key}`).toHaveProperty(key);
      }
    }
  });

  it("every color is a valid hex string", () => {
    const hexRe = /^#[0-9a-fA-F]{6}$/;
    for (const theme of builtinThemes) {
      for (const [key, value] of Object.entries(theme.colors)) {
        expect(value, `${theme.id}.${key} = ${value}`).toMatch(hexRe);
      }
    }
  });
});
