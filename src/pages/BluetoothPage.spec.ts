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

  it("shows a clear message when the adapter is present but no device is paired", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ adapter_present: true, powered: true, devices: [] });
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun périphérique Bluetooth appairé"));
  });

  it("distinguishes a missing bluetoothctl from a machine with no adapter", async () => {
    // Both used to render "Aucun adaptateur Bluetooth détecté" -- one is a
    // package to install, the other is hardware that isn't there.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      adapter_present: false,
      powered: false,
      devices: [],
      tool_error: "bluetoothctl introuvable ou impossible à lancer : … — installez le paquet « bluez »",
    });
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("bluez"));
    expect(wrapper.text()).not.toContain("Aucun adaptateur");
  });

  it("toggles the adapter power and reloads the status", async () => {
    // `Once` variants only: a bare mockImplementation here would outlive
    // this test and break every later one in the file (clearAllMocks
    // resets call history, not the implementation).
    const { invoke } = await import("@tauri-apps/api/core");
    const status = { adapter_present: true, powered: true, devices: [], tool_error: null };
    vi.mocked(invoke)
      .mockResolvedValueOnce(status)
      .mockResolvedValueOnce("Changing power off succeeded")
      .mockResolvedValueOnce({ ...status, powered: false });
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("activé"));
    await wrapper.findAll("button").find((b) => b.text() === "Désactiver")!.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("set_bluetooth_power", { on: false }));
  });

  it("shows an error message instead of a silently blank page when the backend call fails", async () => {
    // Regression guard: get_bluetooth_status is infallible by design, but
    // the IPC call itself was never guarded here (unlike every other page
    // in this app) -- a failure left the page entirely blank (neither the
    // "no adapter" nor the loaded-status branch matches a still-null ref).
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("bluetoothctl introuvable");
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("bluetoothctl introuvable"));
  });
});
