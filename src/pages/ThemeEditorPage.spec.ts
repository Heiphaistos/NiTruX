import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ThemeEditorPage from "./ThemeEditorPage.vue";
import { useStyleStore } from "@/stores/styleStore";
import { useThemeStore } from "@/stores/themeStore";
import { styleRegistry } from "@/styles/registry";
import { builtinThemes } from "@/themes/builtin";

describe("ThemeEditorPage — style tab", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("shows a 'Style' tab button alongside the existing Thème/Disposition tabs", () => {
    const wrapper = mount(ThemeEditorPage);
    expect(wrapper.text()).toContain("Style");
  });

  it("lists all 12 styles when the Style tab is active", async () => {
    const wrapper = mount(ThemeEditorPage);
    const tabs = wrapper.findAll("button");
    const styleTab = tabs.find((b) => b.text() === "Style")!;
    await styleTab.trigger("click");
    for (const style of styleRegistry) {
      expect(wrapper.text()).toContain(style.name);
    }
  });

  it("clicking a style option calls styleStore.setStyle with that style's id", async () => {
    const wrapper = mount(ThemeEditorPage);
    const store = useStyleStore();
    const tabs = wrapper.findAll("button");
    const styleTab = tabs.find((b) => b.text() === "Style")!;
    await styleTab.trigger("click");
    const brutalismOption = wrapper.findAll(".te-style-option").find((el) => el.text().includes("Brutalisme"))!;
    await brutalismOption.trigger("click");
    expect(store.current).toBe("brutalism");
  });
});

describe("ThemeEditorPage — theme tab custom themes", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("shows a saved custom theme as a selectable swatch, not just the builtin ones", () => {
    const themeStore = useThemeStore();
    themeStore.saveCustomTheme({ ...builtinThemes[0], id: "custom-test-1", name: "Mon thème perso" });

    const wrapper = mount(ThemeEditorPage);
    const swatches = wrapper.findAll(".te-swatch");
    expect(swatches.length).toBe(builtinThemes.length + 1);
    expect(swatches.some((s) => s.attributes("title") === "Mon thème perso")).toBe(true);
  });

  it("shows an inline error (not a blocking native alert) when an imported file is invalid", async () => {
    const wrapper = mount(ThemeEditorPage);
    const input = wrapper.find('input[type="file"]');
    const badFile = new File(["not valid json"], "bad-theme.json", { type: "application/json" });
    Object.defineProperty(input.element, "files", { value: [badFile], configurable: true });

    await input.trigger("change");
    await vi.waitFor(() => expect(wrapper.text()).toMatch(/JSON invalide/i));
  });

  it("does not save a custom theme with a blank or whitespace-only name", async () => {
    // Regression guard for the actual gap: handleSave had NO check on
    // themeName at all (not even the flawed bare "" check ScriptsPage.vue
    // had before its own fix) -- a click with an empty or whitespace-only
    // name field would save a swatch with a blank, indistinguishable
    // title, and nothing prevents saving several such blank-named
    // swatches side by side (unlike ScriptsPage's addScript, which at
    // least rejects exact-name duplicates).
    const themeStore = useThemeStore();
    themeStore.setTheme(builtinThemes[0]);

    const wrapper = mount(ThemeEditorPage);
    await wrapper.find(".te-name-input").setValue("   ");
    const saveButton = wrapper.findAll(".te-actions button").find((b) => b.text() === "Sauvegarder")!;
    await saveButton.trigger("click");

    expect(themeStore.customThemes.length).toBe(0);
  });

  it("clicking Sauvegarder activates the saved theme immediately, not just adds it to the unused customThemes list", async () => {
    const themeStore = useThemeStore();
    themeStore.setTheme(builtinThemes[0]); // start on a known theme

    const wrapper = mount(ThemeEditorPage);
    // Edit a color -- mirrors a real user dragging the color picker before saving.
    const colorInput = wrapper.find('input[type="color"]');
    await colorInput.setValue("#123456");

    const saveButton = wrapper.findAll(".te-actions button").find((b) => b.text() === "Sauvegarder")!;
    await saveButton.trigger("click");

    expect(themeStore.active.colors.bgBase).toBe("#123456");
    // The bug: saveCustomTheme alone persists the swatch-list entry but not
    // which theme is "active" -- a reload would revert to builtinThemes[0].
    expect(JSON.parse(localStorage.getItem("nitrux-theme")!).colors.bgBase).toBe("#123456");
  });
});
