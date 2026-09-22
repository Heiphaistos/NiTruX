// src/pages/DiskVisualizerPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import DiskVisualizerPage from "./DiskVisualizerPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
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
});

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
  beforeEach(async () => {
    setActivePinia(createPinia());
    localStorage.clear();
    // Restore the default between tests: the drill-down tests below use
    // mockImplementation, which outlives its own test otherwise.
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(defaultInvokeImpl);
  });

  it("pre-fills the scan directory input from the defaultScanDirectory preference", () => {
    const preferences = usePreferencesStore();
    preferences.setDefaultScanDirectory("/home/dev/downloads");
    const wrapper = mount(DiskVisualizerPage);
    expect((wrapper.find("input").element as HTMLInputElement).value).toBe("/home/dev/downloads");
  });

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

  it("shows the size of each subdirectory and drills into the one that is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string, args) => {
      if (cmd === "get_directory_breakdown") {
        const directory = (args as { directory: string }).directory;
        if (directory === "/home/dev") {
          return Promise.resolve({
            root: "/home/dev",
            total_bytes: 3_221_225_472,
            children: [{ path: "/home/dev/videos", size_bytes: 3_000_000_000 }],
            warning: null,
          });
        }
        return Promise.resolve({
          root: directory,
          total_bytes: 3_000_000_000,
          children: [{ path: `${directory}/vacances`, size_bytes: 2_000_000_000 }],
          warning: null,
        });
      }
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(DiskVisualizerPage);
    await wrapper.find("input").setValue("/home/dev");
    await wrapper.findAll("button").find((b) => b.text() === "Explorer")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("videos"));

    await wrapper.find(".dv-tree-row").trigger("click");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("get_directory_breakdown", { directory: "/home/dev/videos" }),
    );
    await vi.waitFor(() => expect(wrapper.text()).toContain("vacances"));
  });

  it("still shows the sizes when du could not read every subdirectory", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_directory_breakdown") {
        return Promise.resolve({
          root: "/",
          total_bytes: 1_000_000_000,
          children: [{ path: "/var", size_bytes: 900_000_000 }],
          warning: "Certains dossiers n'ont pas pu être lus : du: impossible de lire « /root »",
        });
      }
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(DiskVisualizerPage);
    await wrapper.find("input").setValue("/");
    await wrapper.findAll("button").find((b) => b.text() === "Explorer")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("n'ont pas pu être lus"));
    expect(wrapper.text()).toContain("var");
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
