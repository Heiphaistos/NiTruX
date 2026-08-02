import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import FileToolsPage from "./FileToolsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "verify_file_hash") {
      return Promise.resolve(args?.expected === "matching-hash");
    }
    return Promise.resolve([]);
  }),
}));

describe("FileToolsPage", () => {
  beforeEach(() => vi.clearAllMocks());

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
