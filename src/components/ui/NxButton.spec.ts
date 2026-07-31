import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxButton from "./NxButton.vue";

describe("NxButton", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxButton, { slots: { default: "Enregistrer" } });
    expect(wrapper.text()).toBe("Enregistrer");
  });

  it("defaults to the 'default' variant class", () => {
    const wrapper = mount(NxButton);
    expect(wrapper.classes()).toContain("nx-button--default");
  });

  it.each(["default", "danger", "ghost"] as const)("applies the %s variant class", (variant) => {
    const wrapper = mount(NxButton, { props: { variant } });
    expect(wrapper.classes()).toContain(`nx-button--${variant}`);
  });

  it("emits a click event when clicked and not disabled", async () => {
    const wrapper = mount(NxButton);
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toHaveLength(1);
  });

  it("reflects the disabled prop on the underlying <button> element", () => {
    const wrapper = mount(NxButton, { props: { disabled: true } });
    expect(wrapper.attributes("disabled")).toBeDefined();
  });
});
