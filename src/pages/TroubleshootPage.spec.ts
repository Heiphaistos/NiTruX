// src/pages/TroubleshootPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TroubleshootPage from "./TroubleshootPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("TroubleshootPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the malware/snapshots/troubleshoot tabs, no firewall tab", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Scan malware");
    expect(wrapper.text()).toContain("Snapshots");
    expect(wrapper.text()).toContain("Dépannage");
    expect(wrapper.text()).not.toContain("Pare-feu");
  });

  it("lazy-loads snapshots only the first time the snapshots tab is opened", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const tabs = wrapper.findAll("button");
    const snapshotsTab = tabs.find((b) => b.text() === "Snapshots")!;
    expect(invoke).not.toHaveBeenCalledWith("list_snapshots");
    await snapshotsTab.trigger("click");
    expect(invoke).toHaveBeenCalledWith("list_snapshots");
  });

  it("runs a troubleshoot action via run_troubleshoot_action", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const tabs = wrapper.findAll("button");
    const troubleshootTab = tabs.find((b) => b.text() === "Dépannage")!;
    await troubleshootTab.trigger("click");
    const buttons = wrapper.findAll("button");
    const execButton = buttons.find((b) => b.text() === "Exécuter")!;
    await execButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "clean-cache" });
  });
});
