import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useThemeStore } from "./themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("themeStore.updateActiveColor", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("mutates a single color of the active theme and re-applies to DOM", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[0]);
    store.updateActiveColor("accentPrimary", "#ff00ff");
    expect(store.active.colors.accentPrimary).toBe("#ff00ff");
    expect(document.documentElement.style.getPropertyValue("--nx-accent-primary").trim()).toBe("#ff00ff");
  });
});
