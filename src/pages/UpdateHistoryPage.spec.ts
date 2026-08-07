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

  it("renders every history entry as a distinct row when two share the same start_date", async () => {
    // apt's history.log timestamps have second precision -- a wrapper
    // script running "apt-get update && apt-get upgrade -y" back-to-back
    // (a routine unattended-upgrades-style pattern), or two separate
    // invocations landing in the same second, routinely produces two
    // blocks with an identical Start-Date. Keying the v-for on
    // `e.start_date` alone is not guaranteed unique against this real log
    // shape, the same class of bug already fixed for
    // WiFiAnalyzerPage.vue/TemperaturesPage.vue.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_update_history") {
        return Promise.resolve([
          { start_date: "2026-08-02  01:48:01", commandline: "apt-get update", summary: "" },
          { start_date: "2026-08-02  01:48:01", commandline: "apt-get upgrade -y", summary: "Upgrade: curl:amd64 (7.88.1-9, 7.88.1-10)" },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.findAll(".uh-row").length).toBe(2));
    const commands = wrapper.findAll(".uh-cmd").map((n) => n.text());
    expect(commands).toEqual(["apt-get update", "apt-get upgrade -y"]);
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
