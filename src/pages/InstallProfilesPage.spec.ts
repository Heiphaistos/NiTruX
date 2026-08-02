import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import InstallProfilesPage from "./InstallProfilesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "install_package") {
      if (args?.package === "libreoffice") return Promise.reject("apt: échec");
      return Promise.resolve("ok");
    }
    return Promise.resolve("ok");
  }),
}));

describe("InstallProfilesPage", () => {
  it("lists every profile with its app count", () => {
    const wrapper = mount(InstallProfilesPage);
    expect(wrapper.text()).toContain("Essentiels");
    expect(wrapper.text()).toContain("Développement");
  });

  it("selecting a profile checks all its apps", async () => {
    const wrapper = mount(InstallProfilesPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Essentiels"))!;
    await button.trigger("click");
    const checkboxes = wrapper.findAll("input[type=checkbox]:checked");
    // essentiels has 4 apps
    expect(checkboxes.length).toBe(4);
  });

  it("installs every checked app sequentially and reports a per-app summary, including failures", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(InstallProfilesPage);
    const profileButton = wrapper.findAll("button").find((b) => b.text().includes("Essentiels"))!;
    await profileButton.trigger("click");
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Installer la sélection")!;
    await installButton.trigger("click");
    // Waiting on "Firefox" here would be a trap: it's already in the
    // always-rendered checkbox list from the very first render, so that
    // wait would resolve immediately and NOT actually wait for the
    // sequential install loop to finish -- confirmed by tracing this by
    // hand before writing it (installSelection awaits each installOne in
    // turn; trigger("click") only awaits Vue's nextTick, not the full
    // async handler). "échec" only appears once the libreoffice result
    // (the 2nd of 4, which fails per the mock above) has actually landed
    // in `results`, which is a real completion signal.
    await vi.waitFor(() => expect(wrapper.text()).toContain("échec"));
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "firefox-esr" });
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "libreoffice" });
  });
});
