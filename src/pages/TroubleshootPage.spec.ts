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
    expect(execButtons.length).toBe(4);
    await execButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "fix-broken" });
  });

  it("exposes clean-cache and vacuum-logs, the two actions the backend already validates and implements but had no button for", async () => {
    // security_write.rs::validate_troubleshoot_action accepts 4 actions
    // ("clean-cache" | "fix-broken" | "restart-network" | "vacuum-logs"),
    // and nitrux-pkexec-helper's troubleshoot subcommand implements all 4
    // (apt/dnf/pacman/zypper cache clean, and `journalctl
    // --vacuum-time=7d` respectively) -- but this page only ever exposed
    // 2 of them, leaving two fully-built, already-verified privileged
    // actions with no way for a user to trigger them.
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Vider le cache du gestionnaire de paquets");
    expect(wrapper.text()).toContain("Purger les journaux système de plus de 7 jours");

    const buttons = wrapper.findAll("button");
    const execButtons = buttons.filter((b) => b.text() === "Exécuter");
    await execButtons[2].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "clean-cache" });
    await execButtons[3].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "vacuum-logs" });
  });
});
