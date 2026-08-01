// src/pages/TroubleshootPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TroubleshootPage from "./TroubleshootPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("TroubleshootPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the snapshots/troubleshoot tabs, no malware or firewall tab, defaults to troubleshoot", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Snapshots");
    expect(wrapper.text()).toContain("Dépannage");
    expect(wrapper.text()).not.toContain("Scan malware");
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

  it("runs the fix-broken troubleshoot action via run_troubleshoot_action (troubleshoot is now the default tab)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const buttons = wrapper.findAll("button");
    const execButtons = buttons.filter((b) => b.text() === "Exécuter");
    expect(execButtons.length).toBe(2); // fix-broken, restart-network
    await execButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "fix-broken" });
  });
});
