import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import HardwareInventoryPage from "./HardwareInventoryPage.vue";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const INV = {
  cpu: { title: "Processeur", items: [{ key: "Modèle", value: "AMD Ryzen 7 5800X" }, { key: "Virtualisation matérielle", value: "disponible" }] },
  cpu_cores: [{ key: "cpu0", value: "3800 MHz" }],
  firmware: { title: "Carte mère & firmware", items: [{ key: "TPM", value: "présent — TPM 2.0" }, { key: "Secure Boot", value: "activé" }] },
  memory: { title: "Mémoire", items: [{ key: "Total utilisable", value: "32.0 Go" }] },
  gpus: [{ name: "Radeon RX 6800", details: [{ key: "Pilote", value: "amdgpu" }] }],
  storage: [{ name: "Samsung 980 Pro", details: [{ key: "Périphérique", value: "/dev/nvme0n1" }, { key: "Type", value: "SSD" }] }],
  power: [], network: [], usb: [], audio: [],
};

describe("HardwareInventoryPage", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "get_hardware_inventory") return Promise.resolve(INV);
      if (cmd === "get_firmware_windows_key") return Promise.resolve("XXXXX-YYYYY-ZZZZZ-AAAAA-BBBBB");
      if (cmd === "get_memory_modules") return Promise.resolve({ max_capacity: "128 GB", slots_total: 4, modules: [{ name: "CMK32", details: [{ key: "Type", value: "DDR4" }] }], empty_slots: ["DIMM 1"] });
      if (cmd === "get_smart_report") return Promise.resolve({ device: args!.device, healthy: true, attributes: [{ key: "Usure estimée", value: "3%" }] });
      return Promise.resolve(null);
    });
  });

  it("renders CPU, firmware (TPM/Secure Boot), GPU and storage from the inventory", async () => {
    const wrapper = mount(HardwareInventoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("AMD Ryzen 7 5800X"));
    expect(wrapper.text()).toContain("présent — TPM 2.0");
    expect(wrapper.text()).toContain("Secure Boot");
    expect(wrapper.text()).toContain("Radeon RX 6800");
    expect(wrapper.text()).toContain("Samsung 980 Pro");
  });

  it("reads the firmware OEM key on demand", async () => {
    const wrapper = mount(HardwareInventoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("AMD Ryzen 7 5800X"));
    await wrapper.findAll("button").find((b) => b.text().includes("Clé Windows OEM"))!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("XXXXX-YYYYY-ZZZZZ-AAAAA-BBBBB"));
    expect(invoke).toHaveBeenCalledWith("get_firmware_windows_key");
  });

  it("fetches the per-slot memory detail and a disk's SMART health on demand", async () => {
    const wrapper = mount(HardwareInventoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("32.0 Go"));
    await wrapper.findAll("button").find((b) => b.text().includes("Détail des barrettes"))!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Emplacements libres : DIMM 1"));

    await wrapper.findAll("button").find((b) => b.text().includes("S.M.A.R.T"))!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Sain"));
    expect(invoke).toHaveBeenCalledWith("get_smart_report", { device: "/dev/nvme0n1" });
  });
});
