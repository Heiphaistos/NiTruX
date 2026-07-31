import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxCard from "./NxCard.vue";

describe("NxCard", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxCard, { slots: { default: "<p class=\"probe\">hello</p>" } });
    expect(wrapper.find(".probe").exists()).toBe(true);
    expect(wrapper.find(".probe").text()).toBe("hello");
  });

  it("applies the nx-card base class", () => {
    const wrapper = mount(NxCard);
    expect(wrapper.classes()).toContain("nx-card");
  });

  it("adds a danger modifier class when the danger prop is set", () => {
    const wrapper = mount(NxCard, { props: { danger: true } });
    expect(wrapper.classes()).toContain("nx-card--danger");
  });
});
