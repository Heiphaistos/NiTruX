import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TemperaturesPage from "./TemperaturesPage.vue";

const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
  if (cmd === "get_gpu_snapshot") return Promise.resolve({ nvidia_available: false, gpus: [] });
  return Promise.resolve({
    battery_percent: 80,
    battery_charging: false,
    temperatures: [
      { label: "CPU", celsius: 45.2 },
      { label: "GPU", celsius: 72.8 },
      { label: "NVMe", celsius: 91.0 },
    ],
  });
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(defaultInvokeImpl) }));

describe("TemperaturesPage", () => {
  beforeEach(async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(defaultInvokeImpl);
  });

  it("shows NVIDIA card temperature and utilisation when nvidia-smi is available", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_gpu_snapshot") {
        return Promise.resolve({
          nvidia_available: true,
          gpus: [
            {
              name: "NVIDIA GeForce RTX 3070",
              temperature_celsius: 54,
              utilization_percent: 12,
              memory_used_mb: 1024,
              memory_total_mb: 8192,
            },
          ],
        });
      }
      return defaultInvokeImpl(cmd);
    });
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("RTX 3070"));
    expect(wrapper.text()).toContain("54°C");
    expect(wrapper.text()).toContain("12%");
  });

  it("says no NVIDIA card is present rather than showing an error on AMD/Intel machines", async () => {
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune carte NVIDIA"));
  });
  it("renders one card per sensor with a threshold-colored badge", async () => {
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("CPU"));
    expect(wrapper.text()).toContain("45");
    expect(wrapper.text()).toContain("GPU");
    expect(wrapper.text()).toContain("73");
    expect(wrapper.text()).toContain("NVMe");
    expect(wrapper.text()).toContain("91");
    // CPU (45°C) is under the 60° "success" threshold, NVMe (91°C) is over
    // the 80° "danger" threshold -- both badge classes must appear.
    expect(wrapper.find(".nx-badge--success").exists()).toBe(true);
    expect(wrapper.find(".nx-badge--danger").exists()).toBe(true);
  });

  it("renders every sensor reading as a distinct card when several share the same label", async () => {
    // sensors.rs::read_temperatures builds this list from sysinfo::Components,
    // which surfaces one entry per hwmon component -- on a system with 2+
    // NVMe drives (or multiple hwmon chips reporting a generic sensor name),
    // it is routine for several entries to carry the identical label
    // "Composite" (or "Core 0", etc.), the same real-data-shape risk already
    // recognized for this exact SensorSnapshot.temperatures array on
    // DashboardPage.vue (keyed defensively there via `${t.label}-${i}`) but
    // left as a bare `t.label` key here on the dedicated page -- fixed to
    // match.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      battery_percent: null,
      battery_charging: null,
      temperatures: [
        { label: "Composite", celsius: 42.0 },
        { label: "Composite", celsius: 55.0 },
      ],
    });
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.findAll(".nx-stat-tile").length).toBe(2));
    const values = wrapper.findAll(".nx-stat-tile__value").map((n) => n.text());
    expect(values).toEqual(["42°C", "55°C"]);
  });

  it("shows an empty-state message when no sensors are detected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      battery_percent: null,
      battery_charging: null,
      temperatures: [],
    });
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun capteur"));
  });
});
