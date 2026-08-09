// src/pages/UninstallerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UninstallerPage from "./UninstallerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_installed_packages") {
      return Promise.resolve([
        { name: "curl", version: "7.88.1" },
        { name: "vim", version: "9.0" },
      ]);
    }
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "uninstall_package") return Promise.resolve("désinstallation réussie");
    return Promise.resolve(null);
  }),
}));

describe("UninstallerPage", () => {
  it("lists installed packages on mount and filters by search text", async () => {
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    expect(wrapper.text()).toContain("vim");
    await wrapper.find("input[placeholder*='Rechercher']").setValue("curl");
    expect(wrapper.text()).toContain("curl");
    expect(wrapper.text()).not.toContain("vim");
  });

  it("shows an empty-state message when a search matches no installed package", async () => {
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    await wrapper.find("input[placeholder*='Rechercher']").setValue("zzznotarealpackagezzz");
    expect(wrapper.text()).toContain("Aucun paquet ne correspond à cette recherche.");
  });

  it("keeps the uninstall button disabled until the exact package name is retyped, then calls uninstall_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const uninstallButtons = wrapper.findAll("button").filter((b) => b.text() === "Désinstaller");
    await uninstallButtons[0].trigger("click"); // opens the confirmation row for "curl"
    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la désinstallation")!;
    expect(confirmButton.attributes("disabled")).toBeDefined();
    const confirmInput = wrapper.find("input[placeholder*='curl']");
    await confirmInput.setValue("curl");
    expect(confirmButton.attributes("disabled")).toBeUndefined();
    await confirmButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("uninstall_package", { manager: "apt", package: "curl" }));
  });

  it("shows a clear error instead of silently doing nothing when no native package manager was detected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_installed_packages") return Promise.resolve([{ name: "curl", version: "7.88.1" }]);
      if (cmd === "detect_native_manager") return Promise.resolve(null);
      return Promise.resolve(null);
    });
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const uninstallButton = wrapper.findAll("button").find((b) => b.text() === "Désinstaller")!;
    await uninstallButton.trigger("click");
    const confirmInput = wrapper.find("input[placeholder*='curl']");
    await confirmInput.setValue("curl");
    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la désinstallation")!;
    // Mock call history is shared (not cleared) across tests in this file,
    // so an earlier test's genuine uninstall_package call is still in it --
    // compare the count before/after this click instead of absolute history.
    const uninstallCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "uninstall_package").length;
    await confirmButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("aucun gestionnaire de paquets natif détecté"));
    const uninstallCallsAfter = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "uninstall_package").length;
    expect(uninstallCallsAfter).toBe(uninstallCallsBefore);
  });

  it("shows a clear error and still shows the package list when detect_native_manager rejects", async () => {
    // Regression guard for the actual bug: detect_native_manager had no
    // try/catch -- a rejection there was an unhandled rejection with no
    // error ever shown, and the header was stuck on "Détection du
    // gestionnaire..." forever instead of indicating an actual failure.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_installed_packages") return Promise.resolve([{ name: "curl", version: "7.88.1" }]);
      if (cmd === "detect_native_manager") return Promise.reject("tauri IPC channel closed");
      return Promise.resolve(null);
    });
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("tauri IPC channel closed"));
    expect(wrapper.text()).toContain("curl");
  });
});
