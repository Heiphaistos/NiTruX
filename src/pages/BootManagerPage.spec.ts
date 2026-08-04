import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BootManagerPage from "./BootManagerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    grub: { default_entry: "0", timeout_seconds: "5", distributor: "Debian", cmdline_default: "quiet" },
    efi_entries: [{ id: "0004", label: "debian", is_current: true }],
  }),
}));

describe("BootManagerPage", () => {
  it("shows GRUB defaults and EFI boot entries", async () => {
    const wrapper = mount(BootManagerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("debian"));
    expect(wrapper.text()).toContain("5");
    expect(wrapper.text()).toContain("quiet");
  });

  it("shows a clear message when neither GRUB config nor EFI entries are available", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ grub: null, efi_entries: null });
    const wrapper = mount(BootManagerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("indisponible"));
  });

  it("shows an empty-state message instead of a bare header when efi_entries is an empty array", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      grub: { default_entry: "0", timeout_seconds: "5", distributor: "Debian", cmdline_default: "quiet" },
      efi_entries: [],
    });
    const wrapper = mount(BootManagerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune entrée de démarrage EFI trouvée."));
  });
});
