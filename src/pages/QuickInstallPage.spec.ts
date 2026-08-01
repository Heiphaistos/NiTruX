// src/pages/QuickInstallPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QuickInstallPage from "./QuickInstallPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "install_package") {
      if (args?.package === "fail-me") return Promise.reject("apt: paquet introuvable");
      return Promise.resolve("Installation réussie");
    }
    if (cmd === "install_flatpak_package") return Promise.resolve("Installation Flatpak réussie");
    if (cmd === "install_snap_package") return Promise.resolve("Installation Snap réussie");
    return Promise.resolve(null);
  }),
}));

describe("QuickInstallPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("detects the native manager on mount and renders the catalog", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    expect(invoke).toHaveBeenCalledWith("detect_native_manager");
  });

  it("installs an apt-method app via install_package using the detected manager", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const buttons = wrapper.findAll("button");
    const firefoxButton = buttons.find((b) => b.text() === "Installer" && b.element.closest(".qi-card")?.textContent?.includes("Firefox"))!;
    await firefoxButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "firefox" });
  });

  it("installs a flatpak-method app via install_flatpak_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Discord"));
    const discordCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("Discord"))!;
    const button = discordCard.find("button")!;
    expect(button.attributes("disabled")).toBeUndefined();
    await button.trigger("click");
    await vi.waitFor(() => expect(discordCard.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_flatpak_package", { appId: "com.discordapp.Discord" });
  });

  it("installs a snap-method app via install_snap_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Spotify"));
    const spotifyCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("Spotify"))!;
    const button = spotifyCard.find("button")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(spotifyCard.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_snap_package", { package: "spotify" });
  });

  it("shows an error message when install_package rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "detect_native_manager") return Promise.resolve("apt");
      if (cmd === "install_package" && args?.package === "gimp") return Promise.reject("apt: échec de l'installation");
      return Promise.resolve("ok");
    });
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("GIMP"));
    const gimpCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("GIMP"))!;
    await gimpCard.find("button").trigger("click");
    await vi.waitFor(() => expect(gimpCard.text()).toContain("apt: échec de l'installation"));
  });

  it("filters the catalog by category", async () => {
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const chips = wrapper.findAll(".qi-chip");
    const jeuxChip = chips.find((c) => c.text() === "Jeux")!;
    await jeuxChip.trigger("click");
    expect(wrapper.text()).toContain("Steam");
    expect(wrapper.text()).not.toContain("Firefox");
  });
});
