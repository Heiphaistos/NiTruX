import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSectionHeader from "./NxSectionHeader.vue";

describe("NxSectionHeader", () => {
  it("renders the title", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "Disques & partitions" } });
    expect(wrapper.text()).toContain("Disques & partitions");
  });

  it("renders an optional description when provided", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "T", description: "Une description." } });
    expect(wrapper.text()).toContain("Une description.");
  });

  it("omits the description element entirely when not provided", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "T" } });
    expect(wrapper.find(".nx-section-header__description").exists()).toBe(false);
  });
});
