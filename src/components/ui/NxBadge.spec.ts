import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import NxBadge from "./NxBadge.vue";
import { useThemeStore } from "@/stores/themeStore";
import { builtinThemes } from "@/themes/builtin";
import { contrastRatio, hexToRgb } from "@/lib/accessibleColor";

describe("NxBadge", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("renders default slot content", () => {
    const wrapper = mount(NxBadge, { slots: { default: "Actif" } });
    expect(wrapper.text()).toBe("Actif");
  });

  it.each(["success", "warning", "danger", "info"] as const)("applies the %s status class", (status) => {
    const wrapper = mount(NxBadge, { props: { status }, slots: { default: "x" } });
    expect(wrapper.classes()).toContain(`nx-badge--${status}`);
  });

  it("does not set live-region semantics by default", () => {
    const wrapper = mount(NxBadge, { slots: { default: "x" } });
    expect(wrapper.attributes("role")).toBeUndefined();
    expect(wrapper.attributes("aria-live")).toBeUndefined();
  });

  it("announces the badge to screen readers when live is set", () => {
    const wrapper = mount(NxBadge, { props: { live: true }, slots: { default: "Sauvegarde créée" } });
    expect(wrapper.attributes("role")).toBe("status");
    expect(wrapper.attributes("aria-live")).toBe("polite");
  });

  it.each(["success", "warning", "danger", "info"] as const)(
    "meets WCAG AA contrast (4.5:1) for the %s status on every built-in theme",
    (status) => {
      // Regression guard for the actual bug: the raw theme accent as badge
      // text against its own tinted background failed AA on 31 of 52
      // (theme, status) pairs -- e.g. Adwaita's success badge at 1.98:1,
      // Nord's danger badge at 2.12:1. Verified against the REAL rendered
      // background/color (jsdom-computed rgb()), not recomputed from theme
      // data, so this actually exercises NxBadge's own logic end to end.
      const themeStore = useThemeStore();
      for (const theme of builtinThemes) {
        themeStore.setTheme(theme);
        const wrapper = mount(NxBadge, { props: { status }, slots: { default: "x" } });
        const el = wrapper.element as HTMLElement;
        const bgMatch = el.style.background.match(/rgb\((\d+), (\d+), (\d+)\)/);
        const colorMatch = el.style.color.match(/rgb\((\d+), (\d+), (\d+)\)/);
        expect(bgMatch, `theme ${theme.id}: no background rendered`).not.toBeNull();
        expect(colorMatch, `theme ${theme.id}: no color rendered`).not.toBeNull();
        const bgRgb = bgMatch!.slice(1).map(Number) as [number, number, number];
        const textRgb = colorMatch!.slice(1).map(Number) as [number, number, number];
        expect(contrastRatio(textRgb, bgRgb), `theme ${theme.id}`).toBeGreaterThanOrEqual(4.5);
      }
    },
  );

  it("keeps the badge text recognizably the same hue as the theme's accent color", () => {
    // The fix must not turn every badge the same grey/black -- it should
    // stay a shade of the original accent, just lightened/darkened enough
    // to read. Spot-check hue is preserved (not exact equality, since the
    // whole point is the lightness channel moves).
    const themeStore = useThemeStore();
    themeStore.setTheme(builtinThemes.find((t) => t.id === "adwaita")!); // worst offender, success 1.98:1 before the fix
    const wrapper = mount(NxBadge, { props: { status: "success" }, slots: { default: "x" } });
    const [r, g, b] = (wrapper.element as HTMLElement).style.color.match(/rgb\((\d+), (\d+), (\d+)\)/)!.slice(1).map(Number);
    // Adwaita's accentSuccess (#2ec27e) is green-dominant (g > r, g > b) -- the corrected shade must stay so.
    expect(g).toBeGreaterThan(r);
    expect(g).toBeGreaterThan(b);
  });

  it("contrastRatio computes the known WCAG example correctly (black on white = 21:1)", () => {
    expect(contrastRatio(hexToRgb("#000000"), hexToRgb("#ffffff"))).toBeCloseTo(21, 0);
  });
});
