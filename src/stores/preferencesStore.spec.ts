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
    }));
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("/mnt/data");
    expect(store.dashboardRefreshIntervalMs).toBe(1000);
  });

  it("falls back to defaults when persisted JSON is malformed", () => {
    localStorage.setItem("nitrux-preferences", "not valid json{");
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("");
    expect(store.dashboardRefreshIntervalMs).toBe(2000);
  });
});
