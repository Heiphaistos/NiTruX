import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import SensorsPage from "./SensorsPage.vue";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const CHIPS = [
  { chip: "coretemp", category: "cpu", device: "Intel Core i7", readings: [
    { kind: "temperature", label: "Package", value: 101, unit: "°C", max: 100, critical: 100, alarm: false },
  ] },
  { chip: "nzxt-kraken3", category: "cooling", device: "NZXT Kraken", readings: [
    { kind: "fan", label: "Pompe", value: 2400, unit: "RPM", max: null, critical: null, alarm: false },
    { kind: "temperature", label: "Liquide", value: 34.5, unit: "°C", max: null, critical: null, alarm: false },
  ] },
];

describe("SensorsPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invoke.mockReset();
    invoke.mockResolvedValue(CHIPS);
  });

  it("groups readings by chip and labels each category in French", async () => {
    const wrapper = mount(SensorsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("NZXT Kraken"));
    expect(invoke).toHaveBeenCalledWith("get_all_sensors");
    expect(wrapper.text()).toContain("Processeur");
    expect(wrapper.text()).toContain("Refroidissement / Watercooling");
    expect(wrapper.text()).toContain("2400 RPM");
    expect(wrapper.text()).toContain("34.5 °C");
  });

  it("flags a temperature at its critical threshold as danger", async () => {
    const wrapper = mount(SensorsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Package"));
    expect(wrapper.find(".se-danger").exists()).toBe(true);
  });

  it("shows an empty-state message when nothing is detected", async () => {
    invoke.mockResolvedValue([]);
    const wrapper = mount(SensorsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun capteur détecté"));
  });
});
