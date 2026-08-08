import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import PackagesPage from "./PackagesPage.vue";

// `vi.hoisted` so this default is reachable both from the hoisted
// `vi.mock` factory below and from `beforeEach` -- otherwise a test further
// down that calls `mockImplementation(...)` (not the `Once` variant) would
// permanently replace it for every test running after it, since
// `vi.clearAllMocks()` clears call history but not the implementation
// (same contamination bug class already found and fixed elsewhere).
const defaultInvokeImpl = vi.hoisted(() => () => Promise.resolve([]));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(defaultInvokeImpl),
}));

describe("PackagesPage", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation(defaultInvokeImpl);
  });

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

  it("disables 'Tout mettre à jour' when there are no updates to apply, like its UpdatesPage.vue twin already does", async () => {
    // UpdatesPage.vue (the near-identical sibling page for the same
    // upgrade_all_packages flow) already guards this exact button with
    // `updates.length === 0`, preventing a needless privileged pkexec
    // system-upgrade call when the table is empty. This page's copy of the
    // same button was missing that guard entirely -- clickable even while
    // "Aucune mise à jour disponible." is displayed right above it.
    const wrapper = mount(PackagesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune mise à jour disponible."));
    const upgradeButton = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    expect(upgradeButton.attributes("disabled")).toBeDefined();
  });

  it("re-fetches the update list after a successful upgrade-all, so the table doesn't keep showing packages that were just upgraded", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_updates") return Promise.resolve([{ name: "curl", current_version: "1.0", new_version: "1.1", source: "apt" }]);
      if (cmd === "upgrade_all_packages") return Promise.resolve("2 paquets mis à jour");
      return Promise.resolve(null);
    });
    const wrapper = mount(PackagesPage);
    // Must wait for `updates` to actually be populated (not just for the
    // list_updates call to have been fired) -- the upgrade button is now
    // disabled while updates.length === 0, so clicking too early would
    // silently no-op instead of exercising the upgrade path this test means
    // to cover.
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const listUpdatesCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "list_updates").length;

    const upgradeButton = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await upgradeButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("upgrade_all_packages"));

    const listUpdatesCallsAfter = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "list_updates").length;
    expect(listUpdatesCallsAfter).toBeGreaterThan(listUpdatesCallsBefore);
  });

  it("disables 'Vérifier les mises à jour' while an upgrade is in flight, to avoid racing upgradeAll's own trailing refresh()", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveUpgrade!: (value: string) => void;
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_updates") return Promise.resolve([{ name: "curl", current_version: "1.0", new_version: "1.1", source: "apt" }]);
      if (cmd === "upgrade_all_packages") return new Promise<string>((resolve) => { resolveUpgrade = resolve; });
      return Promise.resolve(null);
    });

    const wrapper = mount(PackagesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));

    const verifyButton = wrapper.findAll("button").find((b) => b.text() === "Vérifier les mises à jour")!;
    expect(verifyButton.attributes("disabled")).toBeUndefined();

    const upgradeButton = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await upgradeButton.trigger("click");
    expect(verifyButton.attributes("disabled")).toBeDefined();

    resolveUpgrade("2 paquets mis à jour");
    await vi.waitFor(() => expect(verifyButton.attributes("disabled")).toBeUndefined());
  });
});
