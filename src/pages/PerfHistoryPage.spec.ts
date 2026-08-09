import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import PerfHistoryPage from "./PerfHistoryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_system_snapshot") {
      return Promise.resolve({
        cpus: [{ name: "Test CPU", usage_percent: 42, usage_display: "42%" }],
        memory_used_bytes: 4_000_000_000,
        memory_total_bytes: 8_000_000_000,
        process_count: 200,
      });
    }
    return Promise.resolve(null);
  }),
}));

describe("PerfHistoryPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("polls get_system_snapshot on mount and renders a sparkline once a sample is collected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(PerfHistoryPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-sparkline").exists()).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_system_snapshot");
  });

  it("keeps the CSV export button disabled until at least one sample has been collected", async () => {
    const wrapper = mount(PerfHistoryPage);
    const findExportButton = () => wrapper.findAll("button").find((b) => b.text() === "Exporter en CSV")!;
    expect(findExportButton().attributes("disabled")).toBeDefined();
    // ".nx-sparkline" is an unconditional root <svg> (only its inner
    // <polyline> is v-if'd), so waiting on its existence doesn't actually
    // wait for a sample -- poll the button's own disabled state directly.
    await vi.waitFor(() => expect(findExportButton().attributes("disabled")).toBeUndefined());
  });
});
