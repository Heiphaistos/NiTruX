// src/components/PkexecIntegrationBanner.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PkexecIntegrationBanner from "./PkexecIntegrationBanner.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "is_pkexec_integration_installed") return Promise.resolve(false);
    return Promise.resolve(null);
  }),
}));

describe("PkexecIntegrationBanner", () => {
  it("shows the banner when integration is not installed", async () => {
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
  });

  it("shows the banner (defaulting to 'needed') when is_pkexec_integration_installed itself rejects", async () => {
    // Regression guard for the actual bug: found live in a browser check,
    // not by reading the source -- with no try/catch around the mounted
    // hook's invoke() call, a rejection left `needed` stuck at `false`
    // forever, silently hiding the only banner that tells an AppImage user
    // their privileged actions are broken and how to fix them.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "is_pkexec_integration_installed") return Promise.reject("IPC layer error");
      return Promise.resolve(null);
    });
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
  });

  it("hides the banner without ever showing a success message when dismissed via 'Plus tard'", async () => {
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
    const dismissButton = wrapper.findAll("button").find((b) => b.text() === "Plus tard")!;
    await dismissButton.trigger("click");
    expect(wrapper.text()).not.toContain("Fonctions privilégiées non activées");
    expect(wrapper.text()).not.toContain("activées avec succès");
  });

  it("calls install_pkexec_integration on click and shows a success message, replacing the banner", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "is_pkexec_integration_installed") return Promise.resolve(false);
      if (cmd === "install_pkexec_integration") return Promise.resolve("installed");
      return Promise.resolve(null);
    });
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Activer les fonctions privilégiées")!;
    await installButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("install_pkexec_integration");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées activées avec succès"));
    expect(wrapper.text()).not.toContain("Fonctions privilégiées non activées");
  });

  it("disables the install button while installing, showing 'Installation...'", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveInstall!: (value: string) => void;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "is_pkexec_integration_installed") return Promise.resolve(false);
      if (cmd === "install_pkexec_integration") return new Promise((resolve) => (resolveInstall = resolve));
      return Promise.resolve(null);
    });
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Activer les fonctions privilégiées")!;
    await installButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Installation..."));
    const buttonWhileInstalling = wrapper.findAll("button").find((b) => b.text() === "Installation...")!;
    expect(buttonWhileInstalling.attributes("disabled")).toBeDefined();

    resolveInstall("installed");
    await vi.waitFor(() => expect(wrapper.text()).toContain("activées avec succès"));
  });

  it("shows an error message and keeps the banner visible (with a retry option) when the install fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "is_pkexec_integration_installed") return Promise.resolve(false);
      if (cmd === "install_pkexec_integration") return Promise.reject("pkexec a échoué (code 127) : polkit refusé");
      return Promise.resolve(null);
    });
    const wrapper = mount(PkexecIntegrationBanner);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Fonctions privilégiées non activées"));
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Activer les fonctions privilégiées")!;
    await installButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("polkit refusé"));
    // The banner (with its retry button) must still be there -- a failed
    // attempt should not silently hide the only way to try again.
    expect(wrapper.text()).toContain("Fonctions privilégiées non activées");
    expect(wrapper.findAll("button").some((b) => b.text() === "Activer les fonctions privilégiées")).toBe(true);
  });
});
