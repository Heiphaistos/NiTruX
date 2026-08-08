// WCAG 2.1 contrast utilities. Extracted here (rather than left inline,
// duplicated per call site) because it's now consumed by real UI logic
// (NxBadge's per-theme text color), not just a one-off verification script.

function srgbToLinear(c: number): number {
  const s = c / 255;
  return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

function relativeLuminance([r, g, b]: [number, number, number]): number {
  return 0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
}

export function contrastRatio(rgb1: [number, number, number], rgb2: [number, number, number]): number {
  const l1 = relativeLuminance(rgb1);
  const l2 = relativeLuminance(rgb2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

export function hexToRgb(hex: string): [number, number, number] {
  const h = hex.replace("#", "");
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16)) as [number, number, number];
}

function rgbToHex([r, g, b]: [number, number, number]): string {
  return "#" + [r, g, b].map((c) => Math.max(0, Math.min(255, Math.round(c))).toString(16).padStart(2, "0")).join("");
}

/** Mirrors CSS `color-mix(in srgb, hex1 pct1%, hex2)` -- plain per-channel sRGB mix. */
export function mixHex(hex1: string, pct1: number, hex2: string): string {
  const [r1, g1, b1] = hexToRgb(hex1);
  const [r2, g2, b2] = hexToRgb(hex2);
  const p = pct1 / 100;
  return rgbToHex([r1 * p + r2 * (1 - p), g1 * p + g2 * (1 - p), b1 * p + b2 * (1 - p)]);
}

function rgbToHsl([r, g, b]: [number, number, number]): [number, number, number] {
  const rN = r / 255;
  const gN = g / 255;
  const bN = b / 255;
  const max = Math.max(rN, gN, bN);
  const min = Math.min(rN, gN, bN);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h: number;
  if (max === rN) h = ((gN - bN) / d + (gN < bN ? 6 : 0)) / 6;
  else if (max === gN) h = ((bN - rN) / d + 2) / 6;
  else h = ((rN - gN) / d + 4) / 6;
  return [h, s, l];
}

function hue2rgb(p: number, q: number, t: number): number {
  let tt = t;
  if (tt < 0) tt += 1;
  if (tt > 1) tt -= 1;
  if (tt < 1 / 6) return p + (q - p) * 6 * tt;
  if (tt < 1 / 2) return q;
  if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6;
  return p;
}

function hslToRgb([h, s, l]: [number, number, number]): [number, number, number] {
  if (s === 0) return [l * 255, l * 255, l * 255];
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  return [hue2rgb(p, q, h + 1 / 3) * 255, hue2rgb(p, q, h) * 255, hue2rgb(p, q, h - 1 / 3) * 255];
}

/**
 * Given a status color and the actual background it will be read against,
 * returns the closest same-hue shade (lightened for a dark background,
 * darkened for a light one) that clears WCAG AA (4.5:1, +0.1 margin) --
 * or the most extreme shade reached if the hue simply can't get there.
 * Falls back to the original color untouched if it already passes, so a
 * theme whose accent is already accessible is never needlessly altered.
 */
export function pickAccessibleTextColor(accentHex: string, backgroundHex: string): string {
  const bgRgb = hexToRgb(backgroundHex);
  const accentRgb = hexToRgb(accentHex);
  if (contrastRatio(accentRgb, bgRgb) >= 4.6) return accentHex;

  const [h, s, l] = rgbToHsl(accentRgb);
  const bgIsDark = relativeLuminance(bgRgb) < 0.5;
  const step = bgIsDark ? 0.005 : -0.005;
  let curL = l;
  for (let i = 0; i < 400; i++) {
    curL += step;
    if (curL > 0.97 || curL < 0.03) break;
    const rgb = hslToRgb([h, s, curL]);
    if (contrastRatio(rgb, bgRgb) >= 4.6) return rgbToHex(rgb);
  }
  return rgbToHex(hslToRgb([h, s, Math.max(0.03, Math.min(0.97, curL))]));
}
