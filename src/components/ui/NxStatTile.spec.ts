import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxStatTile from "./NxStatTile.vue";

describe("NxStatTile", () => {
  it("renders the label and value", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%" } });
    expect(wrapper.text()).toContain("CPU");
    expect(wrapper.text()).toContain("34%");
  });

  it("renders an optional status dot with the given status class when provided", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%", status: "success" } });
    expect(wrapper.find(".nx-stat-tile__dot--success").exists()).toBe(true);
  });

  it("omits the status dot entirely when no status is provided", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%" } });
    expect(wrapper.find(".nx-stat-tile__dot").exists()).toBe(false);
  });
});
