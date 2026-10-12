import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DiskImagePage from "./DiskImagePage.vue";

const invoke = vi.fn();
const open = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: (...a: unknown[]) => open(...a) }));

const DISKS = [
  { path: "/dev/nvme0n1", size_bytes: 512e9, model: "KIOXIA", transport: "nvme" },
  { path: "/dev/sdb", size_bytes: 16e9, model: "SanDisk", transport: "usb" },
];

describe("DiskImagePage", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockImplementation((cmd: string) => Promise.resolve(cmd === "list_whole_disks" ? DISKS : "written"));
    open.mockResolvedValue("/home/u/debian.iso");
  });

  it("offers only USB disks for a bootable key, and writes only after typing the device", async () => {
    const wrapper = mount(DiskImagePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/dev/sdb"));
    expect(wrapper.find("select").text()).not.toContain("/dev/nvme0n1");
    await wrapper.findAll("button").find((b) => b.text() === "Choisir l'ISO…")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/home/u/debian.iso"));
    const write = () => wrapper.findAll("button").find((b) => b.text() === "Écrire sur la clé")!;
    expect(write().attributes("disabled")).toBeDefined();
    await wrapper.find('input[aria-label^="Confirmer"]').setValue("/dev/sdb");
    await write().trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("write_iso_to_usb", { iso: "/home/u/debian.iso", device: "/dev/sdb" }));
    await vi.waitFor(() => expect(wrapper.text()).toContain("Clé prête"));
  });

  it("restore mode lists every disk", async () => {
    const wrapper = mount(DiskImagePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/dev/sdb"));
    await wrapper.findAll("button").find((b) => b.text() === "Restaurer une image")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.find("select").text()).toContain("/dev/nvme0n1"));
  });
});
