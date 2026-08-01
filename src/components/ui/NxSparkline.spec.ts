import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSparkline from "./NxSparkline.vue";

describe("NxSparkline", () => {
  it("renders an svg polyline with one point per value", () => {
    const wrapper = mount(NxSparkline, { props: { values: [10, 50, 30, 80, 20] } });
    const polyline = wrapper.find("polyline");
    expect(polyline.exists()).toBe(true);
    const points = polyline.attributes("points")!.trim().split(" ");
    expect(points.length).toBe(5);
  });

  it("renders an empty svg without error when given no values", () => {
    const wrapper = mount(NxSparkline, { props: { values: [] } });
    expect(wrapper.find("svg").exists()).toBe(true);
    expect(wrapper.find("polyline").exists()).toBe(false);
  });
});
