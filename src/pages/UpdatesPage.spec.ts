// src/pages/UpdatesPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import UpdatesPage from "./UpdatesPage.vue";

// `vi.hoisted` so this default is reachable both from the hoisted
// `vi.mock` factory below and from `beforeEach` -- otherwise a test further
// down that calls `mockImplementation(...)` (not the `Once` variant)
// would permanently replace it for every test running after it, since
// `vi.clearAllMocks()` clears call history but not the implementation
// (same contamination bug class already found and fixed elsewhere).
const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
  if (cmd === "list_updates") {
    return Promise.resolve([
      { name: "curl", current_version: "7.88.1", new_version: "7.89.0", source: "apt" },
    ]);
  }
  if (cmd === "upgrade_all_packages") return Promise.resolve("Mise à jour terminée");
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(defaultInvokeImpl),
}));

describe("UpdatesPage", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation(defaultInvokeImpl);
  });

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
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, ...rest: unknown[]) => {
      if (cmd === "list_updates") return Promise.resolve([]);
      return defaultInvokeImpl(cmd, ...(rest as []));
    });
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune mise à jour"));
  });

  it("disables 'Vérifier' while an upgrade is in flight, to avoid racing upgradeAll's own trailing refresh()", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveUpgrade!: (value: string) => void;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, ...rest: unknown[]) => {
      if (cmd === "upgrade_all_packages") return new Promise<string>((resolve) => { resolveUpgrade = resolve; });
      return defaultInvokeImpl(cmd, ...(rest as []));
    });

    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));

    const verifyButton = wrapper.findAll("button").find((b) => b.text() === "Vérifier")!;
    expect(verifyButton.attributes("disabled")).toBeUndefined();

    const upgradeButton = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await upgradeButton.trigger("click");
    expect(verifyButton.attributes("disabled")).toBeDefined();

    resolveUpgrade("Mise à jour terminée");
    await vi.waitFor(() => expect(verifyButton.attributes("disabled")).toBeUndefined());
  });
});
