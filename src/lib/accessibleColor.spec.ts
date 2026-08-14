import { describe, it, expect } from "vitest";
import { contrastRatio, hexToRgb, mixHex, pickAccessibleTextColor } from "./accessibleColor";

// This is the WCAG-contrast engine NxBadge relies on for every theme (13
// built-ins plus any custom/imported one) -- non-trivial logic (an
// iterative HSL search with a convergence loop and a clamped fallback) that
// had no regression coverage despite every other pure lib in this project
// having one. NxBadge's own comments cite specific measured failures (e.g.
// Adwaita's success badge at 1.98:1) as the reason this exists; these tests
// lock in the guarantee those comments describe instead of relying on a
// one-off manual measurement staying true forever.

describe("contrastRatio", () => {
  it("returns 21:1 for pure black against pure white, the WCAG maximum", () => {
    expect(contrastRatio([0, 0, 0], [255, 255, 255])).toBeCloseTo(21, 1);
  });

  it("returns 1:1 for a color against itself", () => {
    expect(contrastRatio([120, 60, 200], [120, 60, 200])).toBeCloseTo(1, 5);
  });
});

describe("mixHex", () => {
  it("returns hex1 unchanged at 100%", () => {
    expect(mixHex("#ff0000", 100, "#0000ff")).toBe("#ff0000");
  });

  it("returns hex2 unchanged at 0%", () => {
    expect(mixHex("#ff0000", 0, "#0000ff")).toBe("#0000ff");
  });

  it("averages per channel at 50%", () => {
    expect(mixHex("#ff0000", 50, "#0000ff")).toBe("#800080");
  });
});

describe("pickAccessibleTextColor", () => {
  it("returns the original color untouched when it already clears AA", () => {
    // Black on white: already far above the 4.6 (4.5 + margin) threshold.
    expect(pickAccessibleTextColor("#000000", "#ffffff")).toBe("#000000");
  });

  it("guarantees the returned color clears WCAG AA against the given background, even for a known-failing input", () => {
    // Accent equal to its own background: 1:1 contrast, the worst possible
    // input -- the exact class this function exists to fix (NxBadge.vue
    // cites real measured failures like Catppuccin Mocha's success badge
    // at 1.98:1, but the guarantee being tested doesn't depend on which
    // theme produced the failure).
    const accent = "#202020";
    const bg = "#202020";
    expect(contrastRatio(hexToRgb(accent), hexToRgb(bg))).toBe(1);

    const result = pickAccessibleTextColor(accent, bg);
    expect(contrastRatio(hexToRgb(result), hexToRgb(bg))).toBeGreaterThanOrEqual(4.5);
  });

  it("lightens toward a dark background and darkens toward a light one", () => {
    const onDarkBg = hexToRgb(pickAccessibleTextColor("#202020", "#202020"));
    const onLightBg = hexToRgb(pickAccessibleTextColor("#e0e0e0", "#e0e0e0"));
    // Fixing a color against a dark background must move it lighter than
    // it started; against a light background, darker -- never the same
    // adjustment applied blindly regardless of what it's read against.
    expect(onDarkBg.reduce((a, c) => a + c, 0)).toBeGreaterThan(3 * 0x20);
    expect(onLightBg.reduce((a, c) => a + c, 0)).toBeLessThan(3 * 0xe0);
  });
});
