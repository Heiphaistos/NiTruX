// src/pages/ComingSoonPage.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ComingSoonPage from "./ComingSoonPage.vue";

describe("ComingSoonPage", () => {
  it("renders the provided title and phase note", () => {
    const wrapper = mount(ComingSoonPage, { props: { title: "Installation rapide", phase: "Phase R3" } });
    expect(wrapper.text()).toContain("Installation rapide");
    expect(wrapper.text()).toContain("Phase R3");
    expect(wrapper.text()).toContain("Bientôt disponible");
  });
});
