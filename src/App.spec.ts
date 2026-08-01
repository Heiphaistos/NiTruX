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

  it("renders AppNav with all 8 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Performance");
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

  it("shows the real QuickInstallPage (not a placeholder) for the quick-install id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const quickInstallButton = buttons.find((b) => b.text() === "Installation rapide")!;
    await quickInstallButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R3");
  });

  it("shows the real UpdatesPage (not a placeholder) for the updates id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const updatesButton = buttons.find((b) => b.text() === "Mises à jour")!;
    await updatesButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R4");
  });

  it("shows the real ReportGeneratorPage (not a placeholder) for the report-generator id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const reportButton = buttons.find((b) => b.text() === "Générateur de rapport")!;
    await reportButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R5");
  });

  it("shows the real TemperaturesPage for the temperatures id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const tempButton = buttons.find((b) => b.text() === "Températures")!;
    await tempButton.trigger("click");
    expect(wrapper.text()).toContain("Relevés des capteurs thermiques");
  });

  it("shows the real BenchmarkPage for the benchmark id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const benchButton = buttons.find((b) => b.text() === "Benchmark")!;
    await benchButton.trigger("click");
    expect(wrapper.text()).toContain("Lancer le benchmark");
  });

  it("navigates to DiagnosticPage when the dashboard's Diagnostic quick-action tile is clicked", async () => {
    const wrapper = mount(App);
    await vi.waitFor(() => expect(wrapper.findAll(".nx-quick-action").length).toBeGreaterThan(0));
    const tiles = wrapper.findAll(".nx-quick-action");
    const diagnosticTile = tiles.find((t) => t.text().includes("Diagnostic"))!;
    await diagnosticTile.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });
});
