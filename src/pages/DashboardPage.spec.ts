import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import DashboardPage from "./DashboardPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_system_snapshot") {
      return Promise.resolve({
        cpus: [{ name: "Test CPU", usage_percent: 12.5, usage_display: "12.5%" }],
        memory_used_bytes: 4_000_000_000,
        memory_total_bytes: 8_000_000_000,
        process_count: 210,
      });
    }
    if (cmd === "get_sensor_snapshot") {
      return Promise.resolve({ battery_percent: 80, battery_charging: true, temperatures: [] });
    }
    return Promise.resolve(null);
  }),
}));

describe("DashboardPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("polls at the user's configured dashboardRefreshIntervalMs, not a hardcoded value", async () => {
    // The preference is literally named "dashboardRefreshIntervalMs" and
    // its Settings label reads "Intervalle de rafraîchissement du tableau
    // de bord" (dashboard refresh interval) -- but DashboardPage.vue (the
    // actual "Vue d'ensemble" dashboard) never read it, hardcoding 2000ms
    // instead; only PerfHistoryPage.vue (a different page) consumed it.
    const preferences = usePreferencesStore();
    preferences.setDashboardRefreshIntervalMs(9000);
    const setIntervalSpy = vi.spyOn(window, "setInterval");

    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));

    expect(setIntervalSpy).toHaveBeenCalledWith(expect.any(Function), 9000);
    setIntervalSpy.mockRestore();
  });

  it("renders system stats inside NxCard and 5 quick-action tiles", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    expect(wrapper.find(".nx-card").exists()).toBe(true);
    expect(wrapper.text()).toContain("12.5%");
    expect(wrapper.text()).toContain("210");
    const tiles = wrapper.findAll(".nx-quick-action");
    expect(tiles.length).toBe(5);
  });

  it("emits a navigation request when a quick-action tile is clicked", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    const tiles = wrapper.findAll(".nx-quick-action");
    const diagnosticTile = tiles.find((t) => t.text().includes("Diagnostic"))!;
    await diagnosticTile.trigger("click");
    expect(wrapper.emitted("navigate")).toEqual([["diagnostic"]]);
  });

  it("shows memory in French units (Go), not the English 'GB'", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    expect(wrapper.text()).toContain("Go");
    expect(wrapper.text()).not.toContain("GB");
  });

  it("every quick-action gradient stop meets WCAG AA contrast against the tile's white text", async () => {
    // Regression guard for the actual bug: the original gradients (e.g.
    // #fb923c) were 2.26:1 against white, well under the 4.5:1 AA minimum
    // -- computed with the official relative-luminance formula (not
    // eyeballed) rather than trusting the hex values look "dark enough".
    const srgbToLinear = (c: number) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    };
    const relativeLuminance = ([r, g, b]: [number, number, number]) =>
      0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
    const contrastAgainstWhite = (rgb: [number, number, number]) => (1 + 0.05) / (relativeLuminance(rgb) + 0.05);

    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    const tiles = wrapper.findAll(".nx-quick-action");
    expect(tiles.length).toBe(5);
    for (const tile of tiles) {
      // jsdom normalizes `background` to "linear-gradient(135deg, rgb(r, g, b), rgb(r, g, b))",
      // not the original hex -- parse the rgb() triples it actually renders.
      const background = (tile.element as HTMLElement).style.background;
      const stops = [...background.matchAll(/rgb\((\d+), (\d+), (\d+)\)/g)];
      expect(stops.length).toBe(2);
      for (const [, r, g, b] of stops) {
        const rgb: [number, number, number] = [Number(r), Number(g), Number(b)];
        expect(contrastAgainstWhite(rgb)).toBeGreaterThanOrEqual(4.5);
      }
    }
  });
});
