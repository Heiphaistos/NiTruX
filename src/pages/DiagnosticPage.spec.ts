import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DiagnosticPage from "./DiagnosticPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([
    { slot: "00:02.0", class: "VGA", description: "Intel UHD Graphics" },
  ]),
}));

describe("DiagnosticPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_pci_devices and renders the returned device", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DiagnosticPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Intel UHD Graphics"));
    expect(invoke).toHaveBeenCalledWith("get_pci_devices");
  });

  it("renders devices inside an NxCard", async () => {
    const wrapper = mount(DiagnosticPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-card").exists()).toBe(true));
  });
});
