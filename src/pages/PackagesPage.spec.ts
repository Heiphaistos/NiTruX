import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import PackagesPage from "./PackagesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("PackagesPage", () => {
  beforeEach(() => vi.clearAllMocks());

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
});
