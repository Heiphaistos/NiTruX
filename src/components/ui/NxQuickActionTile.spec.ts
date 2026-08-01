// src/components/ui/NxQuickActionTile.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import { Stethoscope } from "lucide-vue-next";
import NxQuickActionTile from "./NxQuickActionTile.vue";

describe("NxQuickActionTile", () => {
  it("renders the icon, label, and applies the given gradient as background", () => {
    const wrapper = mount(NxQuickActionTile, {
      props: { icon: Stethoscope, label: "Diagnostic", gradient: "linear-gradient(135deg,#f97316,#fb923c)" },
    });
    expect(wrapper.text()).toContain("Diagnostic");
    expect(wrapper.find("svg").exists()).toBe(true);
    // jsdom's style-attribute serializer normalizes hex colors to rgb() and
    // adds a space after each comma, so the received attribute is
    // "linear-gradient(135deg, rgb(249, 115, 22), rgb(251, 146, 60))" even
    // though the component was given the hex-based gradient string above.
    expect(wrapper.attributes("style")).toContain("linear-gradient(135deg, rgb(249, 115, 22), rgb(251, 146, 60))");
  });

  it("emits click when clicked", async () => {
    const wrapper = mount(NxQuickActionTile, {
      props: { icon: Stethoscope, label: "Diagnostic", gradient: "linear-gradient(135deg,#f97316,#fb923c)" },
    });
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toBeTruthy();
  });
});
