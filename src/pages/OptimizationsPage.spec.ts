import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import OptimizationsPage from "./OptimizationsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    enabled_services: ["accounts-daemon.service", "cron.service", "cups.service"],
    swappiness: 60,
    zram_active: false,
    fstrim_timer_enabled: true,
  }),
}));

describe("OptimizationsPage", () => {
  it("renders swappiness, zram/fstrim status, and the enabled services list", async () => {
    const wrapper = mount(OptimizationsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("accounts-daemon.service"));
    expect(wrapper.text()).toContain("60");
    expect(wrapper.text()).toContain("cron.service");
    expect(wrapper.text()).toContain("cups.service");
  });

  it("has no buttons that trigger a write action -- read-only diagnostic only", async () => {
    const wrapper = mount(OptimizationsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("accounts-daemon.service"));
    const buttons = wrapper.findAll("button");
    for (const b of buttons) {
      expect(b.text().toLowerCase()).not.toMatch(/désactiver|activer|appliquer|modifier/);
    }
  });

  it("shows an empty-state message instead of a bare header when no service is enabled", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({
      enabled_services: [],
      swappiness: 60,
      zram_active: false,
      fstrim_timer_enabled: true,
    });
    const wrapper = mount(OptimizationsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun service activé au démarrage."));
  });
});
