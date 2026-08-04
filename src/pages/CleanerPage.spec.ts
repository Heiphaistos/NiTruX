import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import CleanerPage from "./CleanerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "get_cache_size_report") {
      return Promise.resolve({ user_cache_bytes: 52_428_800, package_cache_bytes: 104_857_600 });
    }
    if (cmd === "run_troubleshoot_action" && args?.action === "clean-cache") {
      return Promise.resolve("cache vidé");
    }
    if (cmd === "run_troubleshoot_action" && args?.action === "vacuum-logs") {
      return Promise.resolve("journaux purgés");
    }
    return Promise.resolve(null);
  }),
}));

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
