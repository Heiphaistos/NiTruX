// src/pages/DriversPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DriversPage from "./DriversPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    loaded_modules: ["i915", "snd_hda_intel"],
    gpu_driver: "i915 (Intel, open-source)",
    devices: [
      { slot: "00:02.0", description: "Intel Corporation UHD Graphics 620", driver: "i915" },
      { slot: "00:1f.3", description: "Intel Corporation Audio Controller", driver: null },
    ],
  }),
}));

describe("DriversPage", () => {
  it("renders the GPU driver, the per-device table, and an honest note about Linux driver updates", async () => {
    const wrapper = mount(DriversPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("i915 (Intel, open-source)"));
    expect(wrapper.text()).toContain("Intel Corporation UHD Graphics 620");
    expect(wrapper.text()).toContain("i915");
    expect(wrapper.text()).toContain("Intel Corporation Audio Controller");
    expect(wrapper.find(".nx-card").exists()).toBe(true);
    expect(wrapper.text().toLowerCase()).toContain("gestionnaire de paquets");
  });

  it("still lists PCI devices when the kernel has no loadable-module support", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({
      loaded_modules: [],
      gpu_driver: "inconnu",
      devices: [{ slot: "00:02.0", description: "Virtio GPU", driver: "virtio-pci" }],
      devices_error: null,
      modules_error: "ce noyau ne gère pas les modules chargeables (pilotes intégrés au noyau)",
    });
    const wrapper = mount(DriversPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Virtio GPU"));
    expect(wrapper.text()).toContain("ne gère pas les modules chargeables");
  });
});
