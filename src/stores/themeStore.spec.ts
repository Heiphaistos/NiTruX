import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useThemeStore } from "./themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("themeStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.documentElement.removeAttribute("style");
    localStorage.clear();
  });

  it("defaults to the OLED Noir theme", () => {
    const store = useThemeStore();
    expect(store.active.id).toBe("oled-noir");
  });

  it("persists the chosen theme to localStorage and restores it on next init", () => {
    const store = useThemeStore();
    const dracula = builtinThemes.find((t) => t.id === "dracula")!;
    store.setTheme(dracula);
    expect(JSON.parse(localStorage.getItem("nitrux-theme")!).id).toBe("dracula");

    setActivePinia(createPinia());
    const reloaded = useThemeStore();
    expect(reloaded.active.id).toBe("dracula");
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

  it("persists custom themes to localStorage and restores them on next init, same as the active theme", () => {
    const store = useThemeStore();
    const custom = {
      id: "my-custom", name: "My Custom", mode: "dark" as const,
      colors: builtinThemes[0].colors,
    };
    store.saveCustomTheme(custom);

    setActivePinia(createPinia());
    const reloaded = useThemeStore();
    expect(reloaded.customThemes.map((t) => t.id)).toContain("my-custom");
  });

  it("exports the active theme as JSON and re-imports it", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[1]);
    const json = store.exportActiveTheme();
    const imported = store.importTheme(json);
    expect(imported.ok).toBe(true);
    expect(store.customThemes.some((t) => t.id === builtinThemes[1].id)).toBe(true);
  });

  it("activates an imported theme immediately, not just saves it to the unused customThemes list", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[0]); // start on a known, different theme
    const otherTheme = builtinThemes[1];
    expect(store.active.id).not.toBe(otherTheme.id);

    const imported = store.importTheme(JSON.stringify(otherTheme));

    expect(imported.ok).toBe(true);
    expect(store.active.id).toBe(otherTheme.id);
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
