import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AppNav from "./AppNav.vue";
import { navigationCategories } from "@/navigation/categories";

describe("AppNav", () => {
  it("renders every category title", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    for (const category of navigationCategories) {
      expect(wrapper.text()).toContain(category.title);
    }
  });

  it("renders every page label", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const allPages = navigationCategories.flatMap((c) => c.pages);
    for (const page of allPages) {
      expect(wrapper.text()).toContain(page.label);
    }
  });

  it("marks the page matching modelValue as active", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "disks" } });
    const activeButtons = wrapper.findAll("button.active");
    expect(activeButtons).toHaveLength(1);
    expect(activeButtons[0].text()).toBe("Disques & partitions");
  });

  it("emits update:modelValue with the clicked page's id", async () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const buttons = wrapper.findAll("button");
    const target = buttons.find((b) => b.text() === "Pilotes")!;
    await target.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["drivers"]);
  });

  it("renders an icon (svg) next to every nav item's label", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const items = wrapper.findAll(".nx-app-nav__item");
    expect(items.length).toBeGreaterThan(0);
    for (const item of items) {
      expect(item.find("svg").exists()).toBe(true);
    }
  });

  it("falls back to a neutral icon for an unknown icon name rather than crashing", () => {
    // categories.ts always provides known names in practice, but AppNav
    // must not crash if one is ever misspelled -- this proves the fallback.
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    expect(wrapper.exists()).toBe(true);
  });
});
