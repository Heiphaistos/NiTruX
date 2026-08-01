import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BluetoothPage from "./BluetoothPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    adapter_present: true,
    powered: true,
    devices: [{ address: "11:22:33:44:55:66", name: "Sony WH-1000XM4" }],
  }),
}));

describe("BluetoothPage", () => {
  it("shows adapter status and paired devices", async () => {
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Sony WH-1000XM4"));
    expect(wrapper.text()).toContain("activé");
  });

  it("shows a clear message when no adapter is present", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ adapter_present: false, powered: false, devices: [] });
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun adaptateur"));
  });
});
