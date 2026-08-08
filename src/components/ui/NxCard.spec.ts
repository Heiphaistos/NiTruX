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

  it("marks a danger card as an ARIA alert so screen readers announce it when it appears", () => {
    const wrapper = mount(NxCard, { props: { danger: true } });
    expect(wrapper.attributes("role")).toBe("alert");
    expect(wrapper.attributes("aria-live")).toBe("assertive");
  });

  it("does not set alert semantics on a non-danger card", () => {
    const wrapper = mount(NxCard);
    expect(wrapper.attributes("role")).toBeUndefined();
    expect(wrapper.attributes("aria-live")).toBeUndefined();
  });

  it("staticDanger keeps the red styling for a permanent section without alert semantics", () => {
    // Regression guard: role="alert" is discouraged on regions with
    // interactive content (WAI-ARIA authoring practices) -- DisksPage's
    // format-partition section wraps real form controls in a
    // permanently-visible danger card, unlike the transient v-if error
    // messages `danger` is designed for.
    const wrapper = mount(NxCard, { props: { staticDanger: true } });
    expect(wrapper.classes()).toContain("nx-card--danger");
    expect(wrapper.attributes("role")).toBeUndefined();
    expect(wrapper.attributes("aria-live")).toBeUndefined();
  });
});
