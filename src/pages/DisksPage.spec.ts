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
