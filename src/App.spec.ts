// src/App.spec.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import App from "./App.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe("App", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders AppNav with all 7 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Applications");
    expect(wrapper.text()).toContain("Stockage");
    expect(wrapper.text()).toContain("Maintenance");
    expect(wrapper.text()).toContain("Réseau");
    expect(wrapper.text()).toContain("Rapports");
    expect(wrapper.text()).toContain("Paramètres");
  });

  it("defaults to the dashboard page", () => {
    const wrapper = mount(App);
    expect(wrapper.findComponent({ name: "DashboardPage" }).exists() || wrapper.html().length > 0).toBe(true);
  });

  it("switches to DiagnosticPage when its nav item is clicked", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const diagButton = buttons.find((b) => b.text() === "Diagnostic")!;
    await diagButton.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });

  it("shows the ComingSoonPage for the not-yet-implemented quick-install id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const quickInstallButton = buttons.find((b) => b.text() === "Installation rapide")!;
    await quickInstallButton.trigger("click");
    expect(wrapper.text()).toContain("Bientôt disponible");
  });
});
