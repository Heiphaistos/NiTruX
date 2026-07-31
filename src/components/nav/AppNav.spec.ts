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
});
