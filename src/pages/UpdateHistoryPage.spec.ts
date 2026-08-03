import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UpdateHistoryPage from "./UpdateHistoryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_update_history") {
      return Promise.resolve([
        { start_date: "2026-08-02  01:48:01", commandline: "apt-get install -y flatpak", summary: "Install: flatpak:amd64 (1.16.6-1~deb13u1)" },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("UpdateHistoryPage", () => {
  it("lists past update history entries", async () => {
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("apt-get install -y flatpak"));
    expect(wrapper.text()).toContain("2026-08-02");
    expect(wrapper.text()).toContain("flatpak");
  });

  it("shows an empty-state message instead of a blank page when the history is empty", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_update_history") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune mise à jour dans l'historique."));
  });

  it("shows a clear message when history is unavailable for this package manager", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_update_history") return Promise.reject("historique non disponible pour ce gestionnaire de paquets");
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("non disponible"));
  });
});
