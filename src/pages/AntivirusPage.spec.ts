// src/pages/AntivirusPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import AntivirusPage from "./AntivirusPage.vue";

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
  it("scans a directory and lists findings", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    const scanButton = wrapper.findAll("button").find((b) => b.text() === "Scanner")!;
    await scanButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("scan_for_malware", { directory: "/tmp" });
  });

  it("quarantines a finding and removes it from the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    await wrapper.findAll("button").find((b) => b.text() === "Scanner")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));
    const quarantineButton = wrapper.findAll("button").find((b) => b.text().includes("quarantaine"))!;
    await quarantineButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("quarantine_file", { path: "/tmp/evil.sh" });
  });
});
