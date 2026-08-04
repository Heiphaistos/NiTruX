import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import PackagesPage from "./PackagesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("PackagesPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("calls list_updates on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(PackagesPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_updates"));
  });

  it("renders the install form using NxInput and NxSelect", () => {
    const wrapper = mount(PackagesPage);
    expect(wrapper.find(".nx-input").exists()).toBe(true);
    expect(wrapper.find(".nx-select").exists()).toBe(true);
  });

  it("calls install_package with the manager and package name on install click", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(PackagesPage);
    await wrapper.find(".nx-input").setValue("curl");
    const buttons = wrapper.findAll("button");
    const installButton = buttons.find((b) => b.text() === "Installer")!;
    await installButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "curl" });
  });

  it("re-fetches the update list after a successful upgrade-all, so the table doesn't keep showing packages that were just upgraded", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_updates") return Promise.resolve([{ name: "curl", current_version: "1.0", new_version: "1.1", source: "apt" }]);
      if (cmd === "upgrade_all_packages") return Promise.resolve("2 paquets mis à jour");
      return Promise.resolve(null);
    });
    const wrapper = mount(PackagesPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_updates"));
    const listUpdatesCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "list_updates").length;

    const upgradeButton = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await upgradeButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("upgrade_all_packages"));

    const listUpdatesCallsAfter = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "list_updates").length;
    expect(listUpdatesCallsAfter).toBeGreaterThan(listUpdatesCallsBefore);
  });
});
