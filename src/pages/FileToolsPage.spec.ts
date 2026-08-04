import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import FileToolsPage from "./FileToolsPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "verify_file_hash") {
      return Promise.resolve(args?.expected === "matching-hash");
    }
    return Promise.resolve([]);
  }),
}));

describe("FileToolsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("pre-fills both the duplicates and large-files directory inputs from the defaultScanDirectory preference", async () => {
    const preferences = usePreferencesStore();
    preferences.setDefaultScanDirectory("/home/dev/downloads");
    const wrapper = mount(FileToolsPage);
    expect((wrapper.findAll(".nx-input")[0].element as HTMLInputElement).value).toBe("/home/dev/downloads");

    const tabs = wrapper.findAll("button");
    await tabs.find((b) => b.text() === "Gros fichiers")!.trigger("click");
    expect((wrapper.findAll(".nx-input")[0].element as HTMLInputElement).value).toBe("/home/dev/downloads");
  });

  it("shows the 3 tool tabs", () => {
    const wrapper = mount(FileToolsPage);
    expect(wrapper.text()).toContain("Doublons");
    expect(wrapper.text()).toContain("Gros fichiers");
    expect(wrapper.text()).toContain("Vérif. hash");
  });

  it("calls find_duplicate_files with the entered directory on the duplicates tab", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FileToolsPage);
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev");
    const buttons = wrapper.findAll("button");
    const searchButton = buttons.find((b) => b.text() === "Rechercher")!;
    await searchButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("find_duplicate_files", { directory: "/home/dev" });
  });

  it("rejects a non-numeric min-size value on the large-files tab without calling invoke", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FileToolsPage);
    const tabs = wrapper.findAll("button");
    await tabs.find((b) => b.text() === "Gros fichiers")!.trigger("click");
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev");
    await inputs[1].setValue("not-a-number");
    const searchButton = wrapper.findAll("button").find((b) => b.text() === "Rechercher")!;
    await searchButton.trigger("click");
    expect(invoke).not.toHaveBeenCalledWith("find_large_files_cmd", expect.anything());
    expect(wrapper.text()).toContain("taille minimale valide");
  });

  it("accepts a valid min-size value on the large-files tab and calls invoke in bytes", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FileToolsPage);
    const tabs = wrapper.findAll("button");
    await tabs.find((b) => b.text() === "Gros fichiers")!.trigger("click");
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev");
    await inputs[1].setValue("100");
    const searchButton = wrapper.findAll("button").find((b) => b.text() === "Rechercher")!;
    await searchButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("find_large_files_cmd", { directory: "/home/dev", minSizeBytes: 100 * 1024 * 1024 });
  });

  it("verifies a file hash against an expected value and shows a match badge", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FileToolsPage);
    const tabs = wrapper.findAll("button");
    await tabs.find((b) => b.text() === "Vérif. hash")!.trigger("click");
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev/image.iso");
    await inputs[1].setValue("matching-hash");
    const verifyButton = wrapper.findAll("button").find((b) => b.text() === "Vérifier")!;
    await verifyButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("verify_file_hash", { path: "/home/dev/image.iso", algorithm: "sha256", expected: "matching-hash" });
    await vi.waitFor(() => expect(wrapper.text()).toContain("correspond"));
  });

  it("shows a mismatch message when the expected hash does not match", async () => {
    const wrapper = mount(FileToolsPage);
    const tabs = wrapper.findAll("button");
    await tabs.find((b) => b.text() === "Vérif. hash")!.trigger("click");
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev/image.iso");
    await inputs[1].setValue("wrong-hash");
    const verifyButton = wrapper.findAll("button").find((b) => b.text() === "Vérifier")!;
    await verifyButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("ne correspond pas"));
  });
});
