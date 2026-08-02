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
});
