import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DisksPage from "./DisksPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("DisksPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes list_disks and list_disk_usage on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(DisksPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_disks"));
    expect(invoke).toHaveBeenCalledWith("list_disk_usage");
  });

  it("no longer has a tab bar (duplicates/largefiles/hashcheck moved out)", () => {
    const wrapper = mount(DisksPage);
    expect(wrapper.text()).not.toContain("Doublons");
    expect(wrapper.text()).not.toContain("Gros fichiers");
    expect(wrapper.text()).not.toContain("Vérif. hash");
  });

  it("shows empty-state messages instead of blank cards when there are no disks or usage entries", async () => {
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun disque détecté."));
    expect(wrapper.text()).toContain("Aucune information d'utilisation disque.");
  });

  it("keeps the format-partition typed-confirmation gate intact", () => {
    const wrapper = mount(DisksPage);
    expect(wrapper.text()).toContain("Formater une partition");
    const buttons = wrapper.findAll("button");
    const formatButton = buttons.find((b) => b.text().includes("Formater"))!;
    expect(formatButton.attributes("disabled")).toBeDefined();
  });

  it("checks SMART health for a disk and shows the result", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disks") return Promise.resolve([{ name: "sda", size: "500G", partitions: [] }]);
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "get_smart_status") return Promise.resolve({ device: "/dev/sda", health: "PASSED" });
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sda"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Vérifier la santé")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("PASSED"));
    expect(invoke).toHaveBeenCalledWith("get_smart_status", { device: "/dev/sda" });
  });

  it("shows a clear message when SMART is unavailable (e.g. no root)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disks") return Promise.resolve([{ name: "sda", size: "500G", partitions: [] }]);
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "get_smart_status") return Promise.reject("smartctl: Permission denied");
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sda"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Vérifier la santé")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Permission denied"));
  });

  it("keeps a disk's SMART button disabled while it is checking, even after a different disk's check starts too", async () => {
    // smartBusy was a single ref<string | null> holding at most one disk
    // name. Starting a 2nd disk's SMART check while the 1st is still in
    // flight overwrote it, so the 1st disk's button re-evaluated
    // `smartBusy === disk.name` to false and re-enabled while genuinely
    // still checking -- same class of bug already fixed for
    // SystemToolsPage.vue's `running` ref (cycle 182), generalized here.
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveFirst!: (v: unknown) => void;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "list_disks") return Promise.resolve([{ name: "sda", size: "500G", partitions: [] }, { name: "sdb", size: "1T", partitions: [] }]);
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "get_smart_status" && args?.device === "/dev/sda") return new Promise((resolve) => { resolveFirst = resolve; });
      if (cmd === "get_smart_status" && args?.device === "/dev/sdb") return new Promise(() => {}); // never resolves in this test
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sdb"));
    const buttons = wrapper.findAll("button").filter((b) => b.text().includes("Vérifier la santé") || b.text().includes("Vérification..."));
    const [sdaButton, sdbButton] = buttons;
    await sdaButton.trigger("click");
    expect(sdaButton.attributes("disabled")).toBeDefined();

    await sdbButton.trigger("click");
    expect(sdaButton.attributes("disabled")).toBeDefined();

    resolveFirst({ device: "/dev/sda", health: "PASSED" });
    await vi.waitFor(() => expect(wrapper.text()).toContain("PASSED"));
  });

  it("shows disk usage in French units (Go), not the English 'GB'", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disks") return Promise.resolve([]);
      if (cmd === "list_disk_usage") {
        return Promise.resolve([{ mountpoint: "/", total_bytes: 500 * 1024 * 1024 * 1024, used_bytes: 100 * 1024 * 1024 * 1024, used_percent: 20 }]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Go"));
    expect(wrapper.text()).not.toContain("GB");
  });
});
