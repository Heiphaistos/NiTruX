import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import TemperaturesPage from "./TemperaturesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    battery_percent: 80,
    battery_charging: false,
    temperatures: [
      { label: "CPU", celsius: 45.2 },
      { label: "GPU", celsius: 72.8 },
      { label: "NVMe", celsius: 91.0 },
    ],
  }),
}));

describe("TemperaturesPage", () => {
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
