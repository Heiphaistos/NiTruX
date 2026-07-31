import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxBadge from "./NxBadge.vue";

describe("NxBadge", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxBadge, { slots: { default: "Actif" } });
    expect(wrapper.text()).toBe("Actif");
  });

  it.each(["success", "warning", "danger", "info"] as const)("applies the %s status class", (status) => {
    const wrapper = mount(NxBadge, { props: { status }, slots: { default: "x" } });
    expect(wrapper.classes()).toContain(`nx-badge--${status}`);
  });
});
