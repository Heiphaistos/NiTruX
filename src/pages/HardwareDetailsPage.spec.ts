import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import HardwareDetailsPage from "./HardwareDetailsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_hardware_details") {
      return Promise.resolve({
        cpu: { model_name: "11th Gen Intel(R) Core(TM) i5-11400H", sockets: "1", cores_per_socket: "3", threads_per_core: "2" },
        board: { product_name: "Virtual Machine", sys_vendor: "Microsoft Corporation", board_name: null, bios_version: "Hyper-V UEFI Release v4.1" },
        memory: { total_kb: 10231988, available_kb: 6614532, cached_kb: 5031560, swap_total_kb: 2097148 },
      });
    }
    if (cmd === "get_pci_devices") {
      return Promise.resolve([
        { slot: "00:08.0", class: "VGA compatible controller", description: "Microsoft Corporation Hyper-V virtual VGA" },
        { slot: "00:00.0", class: "Host bridge", description: "Intel Corporation Device" },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("HardwareDetailsPage", () => {
  it("shows CPU, board, and memory details", async () => {
    const wrapper = mount(HardwareDetailsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("11th Gen Intel"));
    expect(wrapper.text()).toContain("Virtual Machine");
    expect(wrapper.text()).toContain("Hyper-V UEFI Release v4.1");
  });

  it("shows only GPU-class PCI devices, not every PCI device", async () => {
    const wrapper = mount(HardwareDetailsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Hyper-V virtual VGA"));
    expect(wrapper.text()).not.toContain("Host bridge");
  });

  it("shows an error in the GPU card instead of failing silently when get_pci_devices rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_hardware_details") {
        return Promise.resolve({
          cpu: { model_name: "Test CPU", sockets: "1", cores_per_socket: "1", threads_per_core: "1" },
          board: { product_name: null, sys_vendor: null, board_name: null, bios_version: null },
          memory: { total_kb: 1000, available_kb: 500, cached_kb: 100, swap_total_kb: 0 },
        });
      }
      if (cmd === "get_pci_devices") return Promise.reject("lspci introuvable (paquet requis : pciutils)");
      return Promise.resolve(null);
    });
    const wrapper = mount(HardwareDetailsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("pciutils"));
  });
});
