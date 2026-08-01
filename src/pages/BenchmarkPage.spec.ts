import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BenchmarkPage from "./BenchmarkPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    cpu_hashes_per_sec: 500_000,
    disk_write_mbps: 320.5,
    disk_read_mbps: 480.2,
    memory_bandwidth_gbps: 12.4,
  }),
}));

describe("BenchmarkPage", () => {
  it("runs the benchmark on button click and displays the results", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("320.5"));
    expect(invoke).toHaveBeenCalledWith("run_benchmark");
    expect(wrapper.text()).toContain("480.2");
    expect(wrapper.text()).toContain("12.4");
  });

  it("shows an error message when the benchmark command rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("erreur de benchmark disque");
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("erreur de benchmark disque"));
  });
});
