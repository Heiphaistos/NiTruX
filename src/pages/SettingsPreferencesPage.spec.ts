import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import SettingsPreferencesPage from "./SettingsPreferencesPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

describe("SettingsPreferencesPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("edits the default scan directory via the store", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    await wrapper.find(".nx-input").setValue("/home/dev/Documents");
    expect(store.defaultScanDirectory).toBe("/home/dev/Documents");
  });

  it("changes the dashboard refresh interval via the select", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    await wrapper.find(".nx-select").setValue("5000");
    expect(store.dashboardRefreshIntervalMs).toBe(5000);
  });
});
