// src/pages/FirewallPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import FirewallPage from "./FirewallPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({ active: true, rules: ["22/tcp ALLOW Anywhere"] }),
}));

describe("FirewallPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_firewall_status and renders the active state and rules", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("22/tcp ALLOW Anywhere"));
    expect(invoke).toHaveBeenCalledWith("get_firewall_status");
    expect(wrapper.text()).toContain("actif");
  });

  it("has no tabs (single concern, unlike the old SecurityPage)", () => {
    const wrapper = mount(FirewallPage);
    expect(wrapper.text()).not.toContain("Scan malware");
    expect(wrapper.text()).not.toContain("Dépannage");
  });

  it("shows an empty-state message instead of a blank card when UFW is active with no rules", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ active: true, rules: [] });
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("actif"));
    expect(wrapper.find(".fw-row").exists()).toBe(false);
    expect(wrapper.text()).toMatch(/aucune règle/i);
  });

  it("does not render an empty rules card when UFW is inactive", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ active: false, rules: [] });
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("inactif"));
    expect(wrapper.find(".fw-rules").exists()).toBe(false);
  });

  it("shows the enabled state for a normal user and fetches the rules with authorization on demand", async () => {
    // `ufw status` is root-only: the page used to show a bare error for
    // every non-root user. The backend now falls back to ufw.conf.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ active: true, rules: [], rules_need_privilege: true })
      .mockResolvedValueOnce({ active: true, rules: ["443/tcp ALLOW Anywhere"], rules_need_privilege: false });
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Afficher les règles (admin)"));
    expect(wrapper.text()).toContain("UFW actif");
    expect(wrapper.find(".nx-card--danger").exists()).toBe(false);

    await wrapper.findAll("button").find((b) => b.text() === "Afficher les règles (admin)")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("443/tcp ALLOW Anywhere"));
    expect(invoke).toHaveBeenCalledWith("get_firewall_rules_privileged");
  });

  it("adds a rule from the firewall page itself, then refreshes the rule list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("actif"));
    await wrapper.find('input[aria-label="Port et protocole"]').setValue(" 8080/tcp ");
    await wrapper.findAll("button").find((b) => b.text() === "Autoriser")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Règle ajoutée."));
    expect(invoke).toHaveBeenCalledWith("add_firewall_rule", { portProto: "8080/tcp" });
    expect(invoke).toHaveBeenCalledWith("get_firewall_rules_privileged");
  });

  it("turns an inactive firewall on only after an explicit confirmation", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ active: false, rules: [] })
      .mockResolvedValueOnce({ active: true, rules: ["22/tcp ALLOW Anywhere"] });
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Activer le pare-feu"));
    await wrapper.findAll("button").find((b) => b.text() === "Activer le pare-feu")!.trigger("click");
    expect(invoke).not.toHaveBeenCalledWith("set_firewall_enabled", expect.anything());
    expect(wrapper.text()).toContain("22/tcp");

    await wrapper.findAll("button").find((b) => b.text() === "Confirmer l'activation")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("UFW actif"));
    expect(invoke).toHaveBeenCalledWith("set_firewall_enabled", { enabled: true });
    expect(wrapper.text()).toContain("Désactiver le pare-feu");
  });
});
