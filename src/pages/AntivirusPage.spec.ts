// src/pages/AntivirusPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import AntivirusPage from "./AntivirusPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "scan_for_malware") {
      return Promise.resolve([{ path: "/tmp/evil.sh", signature: "Test.Signature" }]);
    }
    if (cmd === "quarantine_file") return Promise.resolve("mis en quarantaine");
    return Promise.resolve(null);
  }),
}));

describe("AntivirusPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("pre-fills the scan directory input from the defaultScanDirectory preference", () => {
    const preferences = usePreferencesStore();
    preferences.setDefaultScanDirectory("/home/dev/downloads");
    const wrapper = mount(AntivirusPage);
    expect((wrapper.find("input").element as HTMLInputElement).value).toBe("/home/dev/downloads");
  });

  it("scans a directory and lists findings", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    const scanButton = wrapper.findAll("button").find((b) => b.text() === "Scanner")!;
    await scanButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("scan_for_malware", { directory: "/tmp" });
  });

  it("requires typing the file's path before quarantine is possible", async () => {
    // Regression guard for the actual gap: quarantine moves a file out
    // of its original location with no "restore from quarantine" path
    // back in this app -- effectively as irreversible as a permanent
    // delete, and a scanner false positive on a real file is a genuine
    // risk. This used to fire on a single click with zero confirmation,
    // unlike every other comparably irreversible action in this app.
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    await wrapper.findAll("button").find((b) => b.text() === "Scanner")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));

    const startButton = wrapper.findAll("button").find((b) => b.text() === "Mettre en quarantaine")!;
    await startButton.trigger("click");

    expect(invoke).not.toHaveBeenCalledWith("quarantine_file", expect.anything());
    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la mise en quarantaine")!;
    expect(confirmButton.attributes("disabled")).toBeDefined();

    const confirmInputs = wrapper.findAll("input");
    await confirmInputs[confirmInputs.length - 1].setValue("/tmp/evil.sh");
    await confirmButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("quarantine_file", { path: "/tmp/evil.sh" });
  });
});
