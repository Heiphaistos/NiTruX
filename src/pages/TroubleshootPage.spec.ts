// src/pages/TroubleshootPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TroubleshootPage from "./TroubleshootPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("TroubleshootPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the troubleshoot actions with no tabs, no snapshots content", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Réparer les paquets cassés");
    expect(wrapper.text()).toContain("Redémarrer le réseau");
    expect(wrapper.text()).not.toContain("Snapshots");
    expect(wrapper.text()).not.toContain("Créer un instantané");
  });

  it("runs the fix-broken troubleshoot action via run_troubleshoot_action", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const buttons = wrapper.findAll("button");
    const execButtons = buttons.filter((b) => b.text() === "Exécuter");
    expect(execButtons.length).toBe(2);
    await execButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "fix-broken" });
  });
});
