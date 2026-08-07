import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BenchmarkPage from "./BenchmarkPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    cpu_hashes_per_sec: 500_000,
    disk_write_mbps: 320.5,
    disk_read_mbps: 480.2,
    disk_error: null,
    memory_bandwidth_gbps: 12.4,
    cpu_frequency_mhz: 3600,
    disk_health: [
      { device: "/dev/sda", health: "PASSED" },
      { device: "/dev/nvme0n1", health: null },
    ],
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

  it("shows CPU frequency converted to GHz", async () => {
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("3.60 GHz"));
  });

  it("shows per-disk SMART health with a status badge, and 'indisponible' for an unreadable disk", async () => {
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/dev/sda"));
    expect(wrapper.text()).toContain("PASSED");
    expect(wrapper.text()).toContain("/dev/nvme0n1");
    expect(wrapper.text()).toContain("indisponible");
  });

  it("shows 'inconnue' when cpu_frequency_mhz is 0 (sysinfo could not determine it)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({
      cpu_hashes_per_sec: 1,
      disk_write_mbps: 1,
      disk_read_mbps: 1,
      disk_error: null,
      memory_bandwidth_gbps: 1,
      cpu_frequency_mhz: 0,
      disk_health: [],
    });
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("inconnue"));
  });

  it("shows the CPU and memory results plus a disk error, instead of losing everything, when only the disk sub-benchmark fails", async () => {
    // Regression guard for the actual bug: run_benchmark used to fail the
    // whole command (a rejected invoke) the moment the disk sub-benchmark
    // alone failed, discarding the CPU/memory measurements that had
    // already succeeded. Now the backend degrades disk_write_mbps/
    // disk_read_mbps to 0 with disk_error set, and the command itself
    // never rejects for this reason.
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({
      cpu_hashes_per_sec: 500_000,
      disk_write_mbps: 0,
      disk_read_mbps: 0,
      disk_error: "disque plein",
      memory_bandwidth_gbps: 12.4,
      cpu_frequency_mhz: 3600,
      disk_health: [],
    });
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("disque plein"));
    expect(wrapper.text()).toContain("500");
    expect(wrapper.text()).toContain("12.4");
    expect(wrapper.text()).not.toContain("Mo/s");
  });

  it("shows a caveat about the disk measurement possibly reflecting the OS cache rather than real disk throughput", async () => {
    const wrapper = mount(BenchmarkPage);
    // Not shown before a result exists -- there's nothing to caveat yet.
    expect(wrapper.text()).not.toContain("cache du système");
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("cache du système"));
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
