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
});
