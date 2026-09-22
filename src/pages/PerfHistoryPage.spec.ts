import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import PerfHistoryPage from "./PerfHistoryPage.vue";

const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
  if (cmd === "get_system_snapshot") {
    return Promise.resolve({
      cpus: [{ name: "Test CPU", usage_percent: 42, usage_display: "42%" }],
      memory_used_bytes: 4_000_000_000,
      memory_total_bytes: 8_000_000_000,
      process_count: 200,
    });
  }
  if (cmd === "get_perf_history") return Promise.resolve([]);
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(defaultInvokeImpl) }));

describe("PerfHistoryPage", () => {
  beforeEach(async () => {
    setActivePinia(createPinia());
    localStorage.clear();
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(defaultInvokeImpl);
  });

  it("opens with the samples saved by previous sessions instead of an empty graph", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_perf_history") {
        return Promise.resolve([
          { timestamp_ms: 1_726_900_000_000, cpu_percent: 10, memory_percent: 40 },
          { timestamp_ms: 1_726_900_030_000, cpu_percent: 90, memory_percent: 45 },
        ]);
      }
      return defaultInvokeImpl(cmd);
    });
    const wrapper = mount(PerfHistoryPage);
    // The export button unlocks as soon as samples exist -- here that can
    // only come from the stored history, before any new sample lands.
    await vi.waitFor(() =>
      expect(wrapper.findAll("button").find((b) => b.text() === "Exporter en CSV")!.attributes("disabled")).toBeUndefined(),
    );
    expect(invoke).toHaveBeenCalledWith("get_perf_history");
  });

  it("persists each new sample", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(PerfHistoryPage);
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("record_perf_sample", { cpuPercent: 42, memoryPercent: 50 }),
    );
  });

  it("keeps showing the live graphs when saving to disk fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "record_perf_sample" ? Promise.reject("disque plein") : defaultInvokeImpl(cmd),
    );
    const wrapper = mount(PerfHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Historique non enregistré"));
    expect(wrapper.findAll("button").find((b) => b.text() === "Exporter en CSV")!.attributes("disabled")).toBeUndefined();
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
