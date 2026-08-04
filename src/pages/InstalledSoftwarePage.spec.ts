import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import InstalledSoftwarePage from "./InstalledSoftwarePage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_installed_packages") {
      return Promise.resolve([
        { name: "firefox", version: "128.0" },
        { name: "vlc", version: "3.0.20" },
      ]);
    }
    if (cmd === "get_environment_variables") {
      return Promise.resolve([["HOME", "/home/dev"], ["SHELL", "/bin/bash"]]);
    }
    return Promise.resolve(null);
  }),
}));

describe("InstalledSoftwarePage", () => {
  it("lists installed software with version", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("firefox"));
    expect(wrapper.text()).toContain("128.0");
  });

  it("filters the software list by name", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("firefox"));
    const filterInput = wrapper.findAll("input")[0];
    await filterInput.setValue("vlc");
    expect(wrapper.text()).toContain("vlc");
    expect(wrapper.text()).not.toContain("firefox");
  });

  it("shows environment variables", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HOME"));
    expect(wrapper.text()).toContain("/home/dev");
  });

  it("shows an empty-state message when the search matches no package", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("firefox"));
    const filterInput = wrapper.findAll("input")[0];
    await filterInput.setValue("zzznotarealpackagezzz");
    expect(wrapper.text()).toContain("Aucun paquet ne correspond à cette recherche.");
  });

  it("shows an error instead of silently failing when list_installed_packages rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementationOnce(() => Promise.reject("aucun gestionnaire de paquets détecté"));
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("aucun gestionnaire de paquets détecté"));
  });
});
