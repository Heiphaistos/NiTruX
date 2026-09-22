import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import CleanerPage from "./CleanerPage.vue";

// Hoisted so `beforeEach` can restore it: the orphan-cleanup tests below
// use `mockImplementation`, which otherwise outlives its own test and
// breaks every test that runs after it.
const defaultInvokeImpl = vi.hoisted(() => (cmd: string, args?: Record<string, unknown>) => {
  if (cmd === "get_cache_size_report") {
    return Promise.resolve({ user_cache_bytes: 52_428_800, package_cache_bytes: 104_857_600 });
  }
  if (cmd === "run_troubleshoot_action" && args?.action === "clean-cache") {
    return Promise.resolve("cache vidé");
  }
  if (cmd === "run_troubleshoot_action" && args?.action === "vacuum-logs") {
    return Promise.resolve("journaux purgés");
  }
  if (cmd === "list_orphan_configs") return Promise.resolve([]);
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(defaultInvokeImpl) }));

beforeEach(async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockImplementation(defaultInvokeImpl as never);
});

describe("CleanerPage — configurations orphelines", () => {
  it("lists orphan directories and moves the chosen one to the trash", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_cache_size_report") {
        return Promise.resolve({ user_cache_bytes: 1, package_cache_bytes: 1 });
      }
      if (cmd === "list_orphan_configs") {
        return Promise.resolve([
          { path: "/home/dev/.config/skypeforlinux", name: "skypeforlinux", size_bytes: 52_428_800, kind: "configuration" },
        ]);
      }
      if (cmd === "move_to_trash") return Promise.resolve("skypeforlinux");
      return Promise.resolve(null);
    });
    const wrapper = mount(CleanerPage);
    await wrapper.findAll("button").find((b) => b.text() === "Analyser")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("skypeforlinux"));

    await wrapper.findAll("button").find((b) => b.text() === "Mettre à la corbeille")!.trigger("click");
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("move_to_trash", { path: "/home/dev/.config/skypeforlinux" }),
    );
    // The row disappears from the list once trashed, without a rescan.
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun dossier orphelin"));
  });

  it("shows why a cleanup failed instead of silently dropping the row", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_cache_size_report") {
        return Promise.resolve({ user_cache_bytes: 1, package_cache_bytes: 1 });
      }
      if (cmd === "list_orphan_configs") {
        return Promise.resolve([
          { path: "/home/dev/.config/orphan", name: "orphan", size_bytes: 1024, kind: "configuration" },
        ]);
      }
      if (cmd === "move_to_trash") return Promise.reject("corbeille inaccessible : disque plein");
      return Promise.resolve(null);
    });
    const wrapper = mount(CleanerPage);
    await wrapper.findAll("button").find((b) => b.text() === "Analyser")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("orphan"));
    await wrapper.findAll("button").find((b) => b.text() === "Mettre à la corbeille")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("disque plein"));
  });
});

describe("CleanerPage", () => {
  it("loads and displays cache sizes on mount", async () => {
    const wrapper = mount(CleanerPage);
    await vi.waitFor(() => expect(wrapper.text()).toMatch(/50[.,]0 Mo/));
    expect(wrapper.text()).toMatch(/100[.,]0 Mo/);
  });

  it("runs clean-cache when the button is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(CleanerPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Vider le cache"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "clean-cache" }));
  });

  it("runs vacuum-logs when the button is clicked, and does not re-fetch cache sizes (logs aren't part of that report)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(CleanerPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_cache_size_report"));
    const cacheReportCallsBefore = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_cache_size_report").length;

    const button = wrapper.findAll("button").find((b) => b.text().includes("Purger les journaux"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "vacuum-logs" }));
    await vi.waitFor(() => expect(wrapper.text()).toContain("journaux purgés"));

    const cacheReportCallsAfter = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "get_cache_size_report").length;
    expect(cacheReportCallsAfter).toBe(cacheReportCallsBefore);
  });
});
