import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useThemeStore } from "./themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("themeStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.documentElement.removeAttribute("style");
  });

  it("defaults to the first builtin theme", () => {
    const store = useThemeStore();
    expect(store.active.id).toBe(builtinThemes[0].id);
  });

  it("applies theme colors as --nx- CSS custom properties on :root", () => {
    const store = useThemeStore();
    const dracula = builtinThemes.find((t) => t.id === "dracula")!;
    store.setTheme(dracula);
    const root = document.documentElement;
    expect(root.style.getPropertyValue("--nx-bg-base").trim()).toBe(dracula.colors.bgBase);
    expect(root.style.getPropertyValue("--nx-accent-primary").trim()).toBe(dracula.colors.accentPrimary);
  });

  it("registers and lists custom themes", () => {
    const store = useThemeStore();
    const custom = {
      id: "my-custom", name: "My Custom", mode: "dark" as const,
      colors: builtinThemes[0].colors,
    };
    store.saveCustomTheme(custom);
    expect(store.customThemes.map((t) => t.id)).toContain("my-custom");
  });

  it("exports the active theme as JSON and re-imports it", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[1]);
    const json = store.exportActiveTheme();
    const imported = store.importTheme(json);
    expect(imported.ok).toBe(true);
    expect(store.customThemes.some((t) => t.id === builtinThemes[1].id)).toBe(true);
  });

  it("rejects an imported theme whose colors object is missing required keys", () => {
    const store = useThemeStore();
    const incomplete = {
      id: "incomplete-theme",
      name: "Incomplete",
      mode: "dark" as const,
      colors: {
        bgBase: "#111111",
        bgElevated: "#222222",
        // missing: bgOverlay, border, textPrimary, textSecondary, accentPrimary,
        // accentSecondary, accentSuccess, accentWarning, accentDanger
      },
    };
    const result = store.importTheme(JSON.stringify(incomplete));
    expect(result.ok).toBe(false);
    expect(store.customThemes.some((t) => t.id === "incomplete-theme")).toBe(false);
  });
});
