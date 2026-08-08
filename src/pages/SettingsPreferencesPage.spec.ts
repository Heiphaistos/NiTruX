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

  it("edits the CPU/RAM/disk alert thresholds via the store", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    const inputs = wrapper.findAll(".pref-threshold-input");
    expect(inputs.length).toBe(3);

    await inputs[0].setValue("90");
    await inputs[0].trigger("change");
    expect(store.cpuAlertThreshold).toBe(90);

    await inputs[1].setValue("70");
    await inputs[1].trigger("change");
    expect(store.ramAlertThreshold).toBe(70);

    await inputs[2].setValue("95");
    await inputs[2].trigger("change");
    expect(store.diskAlertThreshold).toBe(95);
  });

  it("does not call the store with NaN when a threshold field is cleared mid-edit", async () => {
    // Regression guard: clearing the field to type a new value fires a
    // "change" event with an empty string -- Number("") is NaN, which must
    // not clobber the last valid threshold with a NaN write.
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    const cpuInput = wrapper.findAll(".pref-threshold-input")[0];
    await cpuInput.setValue("");
    await cpuInput.trigger("change");
    expect(store.cpuAlertThreshold).toBe(80);
  });
});
