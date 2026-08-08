import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { usePreferencesStore } from "./preferencesStore";

describe("preferencesStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("defaults to sensible values when nothing persisted", () => {
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("");
    expect(store.dashboardRefreshIntervalMs).toBe(2000);
    expect(store.cpuAlertThreshold).toBe(80);
    expect(store.ramAlertThreshold).toBe(80);
    expect(store.diskAlertThreshold).toBe(85);
  });

  it("setDefaultScanDirectory updates state and persists", () => {
    const store = usePreferencesStore();
    store.setDefaultScanDirectory("/home/dev");
    expect(store.defaultScanDirectory).toBe("/home/dev");
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!).defaultScanDirectory).toBe("/home/dev");
  });

  it("setDashboardRefreshIntervalMs updates state and persists", () => {
    const store = usePreferencesStore();
    store.setDashboardRefreshIntervalMs(5000);
    expect(store.dashboardRefreshIntervalMs).toBe(5000);
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!).dashboardRefreshIntervalMs).toBe(5000);
  });

  it("reads persisted preferences on store creation", () => {
    localStorage.setItem("nitrux-preferences", JSON.stringify({
      defaultScanDirectory: "/mnt/data",
      dashboardRefreshIntervalMs: 1000,
      cpuAlertThreshold: 90,
      ramAlertThreshold: 70,
      diskAlertThreshold: 95,
    }));
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("/mnt/data");
    expect(store.dashboardRefreshIntervalMs).toBe(1000);
    expect(store.cpuAlertThreshold).toBe(90);
    expect(store.ramAlertThreshold).toBe(70);
    expect(store.diskAlertThreshold).toBe(95);
  });

  it("falls back to defaults when persisted JSON is malformed", () => {
    localStorage.setItem("nitrux-preferences", "not valid json{");
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("");
    expect(store.dashboardRefreshIntervalMs).toBe(2000);
  });

  it("falls back to defaults when persisted JSON is missing the new threshold fields (pre-upgrade shape)", () => {
    localStorage.setItem("nitrux-preferences", JSON.stringify({
      defaultScanDirectory: "/mnt/data",
      dashboardRefreshIntervalMs: 1000,
    }));
    const store = usePreferencesStore();
    expect(store.cpuAlertThreshold).toBe(80);
  });

  it.each([
    ["setCpuAlertThreshold", "cpuAlertThreshold"] as const,
    ["setRamAlertThreshold", "ramAlertThreshold"] as const,
    ["setDiskAlertThreshold", "diskAlertThreshold"] as const,
  ])("%s updates state, persists, and clamps to 1-100", (action, field) => {
    const store = usePreferencesStore();
    store[action](65);
    expect(store[field]).toBe(65);
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!)[field]).toBe(65);
  });

  it.each([
    ["setCpuAlertThreshold", "cpuAlertThreshold"] as const,
    ["setRamAlertThreshold", "ramAlertThreshold"] as const,
    ["setDiskAlertThreshold", "diskAlertThreshold"] as const,
  ])("%s clamps an out-of-range or non-finite value instead of persisting it as-is", (action, field) => {
    // Regression guard: the <input type="number" min="1" max="100">
    // attributes are decorative only -- v-model.number never enforces
    // them, so a field cleared to "" (NaN via v-model.number) or typed as
    // 0/150 must still be clamped here, not trusted as-is. This exact bug
    // hit NiTriTe Windows twice before it started clamping on save.
    const store = usePreferencesStore();
    store[action](0);
    expect(store[field]).toBe(1);
    store[action](150);
    expect(store[field]).toBe(100);
    store[action](NaN);
    expect(store[field]).toBeGreaterThanOrEqual(1);
  });
});
