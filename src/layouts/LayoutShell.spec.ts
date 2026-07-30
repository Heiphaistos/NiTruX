import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import { useLayoutStore } from "@/stores/layoutStore";
import { layoutRegistry } from "./registry";
import LayoutShell from "./LayoutShell.vue";

describe("LayoutShell", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders default slot content under sidebar-classic", () => {
    const wrapper = mount(LayoutShell, {
      slots: { default: "<div class=\"probe\">content</div>", nav: "<div class=\"nav-probe\" />" },
    });
    expect(wrapper.find(".probe").exists()).toBe(true);
    expect(wrapper.find(".nav-probe").exists()).toBe(true);
  });

  it.each(layoutRegistry.map((l) => l.id))("renders default slot content under %s", async (id) => {
    const store = useLayoutStore();
    store.setLayout(id);
    const wrapper = mount(LayoutShell, {
      slots: { default: "<div class=\"probe\">content</div>", nav: "<div class=\"nav-probe\" />" },
    });
    expect(wrapper.find(".probe").exists()).toBe(true);
  });

  it("falls back to sidebar-classic when current holds an invalid layout id", () => {
    const store = useLayoutStore();
    // Simulate corrupted/edited localStorage bypassing setLayout's type-checked signature.
    store.current = "not-a-real-layout" as unknown as typeof store.current;
    const wrapper = mount(LayoutShell, {
      slots: { default: "<div class=\"probe\">content</div>", nav: "<div class=\"nav-probe\" />" },
    });
    expect(wrapper.find(".probe").exists()).toBe(true);
    expect(wrapper.find(".nx-layout--sidebar-classic").exists()).toBe(true);
  });
});
