// src/pages/DiskVisualizerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DiskVisualizerPage from "./DiskVisualizerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_disk_usage") {
      return Promise.resolve([
        { mountpoint: "/", total_bytes: 100_000_000_000, used_bytes: 42_000_000_000, used_percent: 42 },
      ]);
    }
    if (cmd === "find_large_files_cmd") {
      return Promise.resolve([
        { path: "/home/dev/big.iso", size_bytes: 4_000_000_000 },
        { path: "/home/dev/small.iso", size_bytes: 1_000_000_000 },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("DiskVisualizerPage", () => {
  it("shows per-mountpoint usage bars on mount", async () => {
    const wrapper = mount(DiskVisualizerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/"));
    expect(wrapper.text()).toContain("42%");
  });

  it("scans a directory for large files and shows them sorted by size, largest first", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DiskVisualizerPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Analyser")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("big.iso"));
    expect(invoke).toHaveBeenCalledWith("find_large_files_cmd", { directory: "/home/dev", minSizeBytes: 104_857_600 });
    const paths = wrapper.findAll(".dv-file-path").map((n) => n.text());
    expect(paths).toEqual(["/home/dev/big.iso", "/home/dev/small.iso"]);
  });

  it("shows an empty-state message when a scan completes with no large files found", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "find_large_files_cmd") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(DiskVisualizerPage);
    await wrapper.find("input").setValue("/home/dev/empty");
    const button = wrapper.findAll("button").find((b) => b.text() === "Analyser")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toMatch(/aucun gros fichier/i));
    expect(wrapper.find(".dv-file-row").exists()).toBe(false);
  });
});
