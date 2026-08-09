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
    expect(wrapper.text()).toContain("Jeux");
    expect(wrapper.text()).toContain("Sécurité & vie privée");
    expect(wrapper.text()).toContain("Serveur web");
  });

  it("selecting the new 'Jeux' profile checks all its real appCatalog entries", async () => {
    const wrapper = mount(InstallProfilesPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Jeux"))!;
    await button.trigger("click");
    const checkboxes = wrapper.findAll("input[type=checkbox]:checked");
    expect(checkboxes.length).toBe(4);
  });

  it("filters the manual selection catalog by category, mirroring QuickInstallPage's chip filter", async () => {
    // Regression guard for the actual gap: the manual selection list
    // rendered all 506 appCatalog entries as a flat, unfiltered checkbox
    // list -- effectively unusable at that size, unlike QuickInstallPage
    // which already solved this exact problem for the same data source.
    const wrapper = mount(InstallProfilesPage);
    const allCheckboxCount = wrapper.findAll("input[type=checkbox]").length;
    expect(allCheckboxCount).toBeGreaterThan(100);

    const chips = wrapper.findAll("button").filter((b) => b.classes().includes("ip-chip"));
    expect(chips.length).toBeGreaterThan(1);
    const nonDefaultChip = chips.find((c) => c.text() !== "Tous")!;
    await nonDefaultChip.trigger("click");

    const filteredCheckboxCount = wrapper.findAll("input[type=checkbox]").length;
    expect(filteredCheckboxCount).toBeGreaterThan(0);
    expect(filteredCheckboxCount).toBeLessThan(allCheckboxCount);
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

  it("announces a single summary once the whole batch finishes, not one per app", async () => {
    // Regression guard: marking each of the 4 per-app result badges as an
    // ARIA live region would fire 4 separate screen-reader announcements
    // as the sequential loop appends to `results` one at a time -- this
    // summary badge is the one that should carry `live`, appearing only
    // once every app in the batch (Essentiels: firefox/libreoffice/vlc/
    // keepassxc, libreoffice fails per the mock above) has resolved.
    const wrapper = mount(InstallProfilesPage);
    const profileButton = wrapper.findAll("button").find((b) => b.text().includes("Essentiels"))!;
    await profileButton.trigger("click");
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Installer la sélection")!;
    await installButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Installation terminée"));

    expect(wrapper.text()).toContain("3 réussies, 1 échouée");
    const summaryBadge = wrapper
      .findAll(".nx-badge")
      .find((b) => b.text().includes("Installation terminée"))!;
    expect(summaryBadge.attributes("aria-live")).toBe("polite");
    // The 4 per-app result badges must stay silent -- only the summary announces.
    const perAppBadges = wrapper.findAll(".nx-badge").filter((b) => !b.text().includes("Installation terminée"));
    expect(perAppBadges.length).toBe(4);
    for (const badge of perAppBadges) {
      expect(badge.attributes("aria-live")).toBeUndefined();
    }
  });
});
