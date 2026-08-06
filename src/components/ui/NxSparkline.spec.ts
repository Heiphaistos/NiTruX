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

  it("scales responsively to its container instead of having a fixed pixel width", () => {
    // Regression guard: a bare `:width`/`:height` SVG attribute pair never
    // shrinks below its intrinsic pixel size, overflowing a narrower card
    // or window (this app's window has no minWidth in tauri.conf.json, so
    // it can be resized well below a 600px chart). The svg must instead
    // use `width: 100%` (fluid) with a `viewBox` (so the plotted points
    // still scale correctly), capped by its own `width` prop as a
    // max-width rather than a hard attribute.
    const wrapper = mount(NxSparkline, { props: { values: [1, 2, 3], width: 600, height: 80 } });
    const svg = wrapper.find("svg");
    expect(svg.attributes("width")).toBeUndefined();
    expect(svg.attributes("viewBox")).toBe("0 0 600 80");
    expect(svg.attributes("height")).toBe("80");
  });
});
