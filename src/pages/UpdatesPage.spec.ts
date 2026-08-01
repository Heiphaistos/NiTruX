// src/pages/UpdatesPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import UpdatesPage from "./UpdatesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_updates") {
      return Promise.resolve([
        { name: "curl", current_version: "7.88.1", new_version: "7.89.0", source: "apt" },
      ]);
    }
    if (cmd === "upgrade_all_packages") return Promise.resolve("Mise à jour terminée");
    return Promise.resolve(null);
  }),
}));

describe("UpdatesPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("loads and displays upgradable packages on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    expect(invoke).toHaveBeenCalledWith("list_updates");
    expect(wrapper.text()).toContain("7.89.0");
  });

  it("calls upgrade_all_packages when the button is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("upgrade_all_packages"));
  });

  it("shows an empty-state message when there are no updates", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_updates") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune mise à jour"));
  });
});
