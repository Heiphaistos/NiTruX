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

  it("keeps the format-partition typed-confirmation gate intact", () => {
    const wrapper = mount(DisksPage);
    expect(wrapper.text()).toContain("Formater une partition");
    const buttons = wrapper.findAll("button");
    const formatButton = buttons.find((b) => b.text().includes("Formater"))!;
    expect(formatButton.attributes("disabled")).toBeDefined();
  });
});
