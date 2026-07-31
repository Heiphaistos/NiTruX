import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ThemeEditorPage from "./ThemeEditorPage.vue";
import { useStyleStore } from "@/stores/styleStore";
import { styleRegistry } from "@/styles/registry";

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
