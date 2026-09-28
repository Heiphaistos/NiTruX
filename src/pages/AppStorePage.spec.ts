import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import AppStorePage from "./AppStorePage.vue";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const SOURCES = { native: "apt", flatpak: false, flathub: false, snap: true, snapd_running: true };
const RESULTS = {
  results: [
    { source: "native", manager: "apt", id: "vlc", name: "vlc", summary: "multimedia player", version: "" },
    { source: "snap", manager: null, id: "vlc", name: "vlc", summary: "The ultimate media player", version: "3.0.20" },
  ],
  errors: ["flatpak : délai dépassé"],
};

describe("AppStorePage", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "get_install_sources") return Promise.resolve(SOURCES);
      if (cmd === "list_installed_packages") return Promise.resolve([{ name: "vlc", version: "3" }]);
      if (cmd === "search_packages") return Promise.resolve(RESULTS);
      if (cmd === "install_snap_package") return Promise.resolve("vlc 3.0.20 installé");
      if (cmd === "setup_flatpak") return Promise.resolve("Flatpak et Flathub sont prêts.");
      return Promise.resolve(null);
    });
  });

  it("shows each installation source and offers to enable a missing one", async () => {
    const wrapper = mount(AppStorePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Snap Store prêt"));
    expect(wrapper.text()).toContain("apt");
    const btn = wrapper.findAll("button").find((b) => b.text() === "Activer Flatpak + Flathub")!;
    await btn.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Flatpak et Flathub sont prêts."));
    expect(invoke).toHaveBeenCalledWith("setup_flatpak");
  });

  it("searches every source, keeps per-source errors visible, and installs from the chosen one", async () => {
    const wrapper = mount(AppStorePage);
    await wrapper.find("input").setValue("vlc");
    await wrapper.findAll("button").find((b) => b.text() === "Rechercher")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("The ultimate media player"));
    expect(invoke).toHaveBeenCalledWith("search_packages", { query: "vlc" });
    expect(wrapper.text()).toContain("flatpak : délai dépassé");

    const snapCard = wrapper.findAll(".st-result").find((c) => c.text().includes("Snap"))!;
    await snapCard.findAll("button").find((b) => b.text() === "Installer")!.trigger("click");
    await vi.waitFor(() => expect(snapCard.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_snap_package", { package: "vlc" });
  });

  it("filters results by source", async () => {
    const wrapper = mount(AppStorePage);
    await wrapper.find("input").setValue("vlc");
    await wrapper.findAll("button").find((b) => b.text() === "Rechercher")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.findAll(".st-result")).toHaveLength(2));
    await wrapper.findAll(".st-chip").find((c) => c.text().startsWith("Snap"))!.trigger("click");
    expect(wrapper.findAll(".st-result")).toHaveLength(1);
  });

  it("marks a native result already installed instead of offering it", async () => {
    const wrapper = mount(AppStorePage);
    await wrapper.find("input").setValue("vlc");
    await wrapper.findAll("button").find((b) => b.text() === "Rechercher")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("multimedia player"));
    const nativeCard = wrapper.findAll(".st-result").find((c) => c.text().includes("apt"))!;
    expect(nativeCard.text()).toContain("Déjà installé");
  });
});
