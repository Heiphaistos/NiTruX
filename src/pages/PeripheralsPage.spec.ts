import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PeripheralsPage from "./PeripheralsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_monitors") return Promise.resolve(["Virtual-1"]);
    if (cmd === "get_usb_devices") return Promise.resolve(["Bus 001 Device 001: ID 1d6b:0002 Linux Foundation"]);
    if (cmd === "get_audio_sinks") return Promise.resolve([{ name: "auto_null", driver: "PipeWire", state: "SUSPENDED" }]);
    if (cmd === "get_printers") return Promise.resolve([]);
    return Promise.resolve(null);
  }),
}));

describe("PeripheralsPage", () => {
  it("shows monitors, USB devices, and audio sinks", async () => {
    const wrapper = mount(PeripheralsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Virtual-1"));
    expect(wrapper.text()).toContain("Linux Foundation");
    expect(wrapper.text()).toContain("auto_null");
  });

  it("shows a clear message when no printers are detected", async () => {
    const wrapper = mount(PeripheralsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune imprimante détectée"));
  });

  it("shows an error message for a failing section without blocking the other three", async () => {
    // Regression guard: none of the 4 invoke() calls here were ever
    // guarded by try/catch -- a single failing section (e.g. no CUPS, so
    // get_printers rejects) previously left ALL FOUR sections' data unset,
    // since an unhandled rejection in one branch of the sequential
    // onMounted body aborted the rest.
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_monitors") return Promise.resolve(["Virtual-1"]);
      if (cmd === "get_usb_devices") return Promise.resolve(["Bus 001 Device 001: ID 1d6b:0002 Linux Foundation"]);
      if (cmd === "get_audio_sinks") return Promise.resolve([{ name: "auto_null", driver: "PipeWire", state: "SUSPENDED" }]);
      if (cmd === "get_printers") return Promise.reject("lpstat introuvable");
      return Promise.resolve(null);
    });
    const wrapper = mount(PeripheralsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("lpstat introuvable"));
    // The other three sections must still have loaded successfully.
    expect(wrapper.text()).toContain("Virtual-1");
    expect(wrapper.text()).toContain("Linux Foundation");
    expect(wrapper.text()).toContain("auto_null");
  });

  it("does not show 'no monitor detected' while get_monitors is still pending", async () => {
    // monitors/usbDevices/audioSinks default to [] (not null), so the
    // "length === 0" empty-state check is indistinguishable from "still
    // loading" -- unlike `printers`, which uses a null sentinel in this
    // same file specifically to avoid this. Delaying the mock's
    // resolution exposes the premature message.
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveMonitors!: (value: string[]) => void;
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_monitors") return new Promise((resolve) => (resolveMonitors = resolve));
      if (cmd === "get_usb_devices") return Promise.resolve([]);
      if (cmd === "get_audio_sinks") return Promise.resolve([]);
      if (cmd === "get_printers") return Promise.resolve([]);
      return Promise.resolve(null);
    });

    const wrapper = mount(PeripheralsPage);
    await wrapper.vm.$nextTick();
    expect(wrapper.text()).not.toContain("Aucun moniteur détecté.");

    resolveMonitors(["Virtual-1"]);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Virtual-1"));
  });
});
