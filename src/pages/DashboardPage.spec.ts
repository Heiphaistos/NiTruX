import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import DashboardPage from "./DashboardPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

// `vi.hoisted` so this is reachable both from the hoisted `vi.mock` factory
// below AND from `beforeEach` -- a later test overriding `invoke`'s
// implementation with `mockImplementation(...)` (to simulate a rejected
// list_disk_usage call, see "does not show a health score...") otherwise
// permanently replaces it for every test that runs afterward in this file,
// since `mockImplementation` has no automatic per-test scope. Reproduced
// live: adding tests after that one that rely on the default
// list_disk_usage behavior failed, not because of anything wrong in
// DashboardPage.vue itself, but because they silently inherited the prior
// test's rejection instead of the default resolved value.
const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
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
  if (cmd === "list_disk_usage") {
    return Promise.resolve([
      { mountpoint: "/", total_bytes: 100_000_000_000, used_bytes: 20_000_000_000, used_percent: 20 },
      { mountpoint: "/boot", total_bytes: 1_000_000_000, used_bytes: 900_000_000, used_percent: 90 },
    ]);
  }
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(defaultInvokeImpl),
}));

describe("DashboardPage", () => {
  beforeEach(async () => {
    setActivePinia(createPinia());
    localStorage.clear();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation(defaultInvokeImpl);
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

  it("computes the system health score from the root (/) mountpoint, not just the first disk entry", async () => {
    // The mock's first list_disk_usage entry is "/boot" at 90% (would tank
    // the score if the code naively used disks[0]) -- the real "/" entry
    // (20%, listed second) is what the score must actually key off.
    // CPU 12.5% (<30 -> 34pts) + RAM 50% used/50% free (not >50%, falls to
    // the >25% tier -> 16pts) + disk 20% (<80 -> 33pts) = 83.
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Score système"));
    expect(wrapper.text()).toContain("83");
    expect(wrapper.text()).toContain("Excellent");
  });

  it("does not show a health score while disk usage is still unknown", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_system_snapshot") {
        return Promise.resolve({
          cpus: [{ name: "Test CPU", usage_percent: 12.5, usage_display: "12.5%" }],
          memory_used_bytes: 4_000_000_000,
          memory_total_bytes: 8_000_000_000,
          process_count: 210,
        });
      }
      if (cmd === "list_disk_usage") return Promise.reject("lsblk introuvable");
      return Promise.resolve(null);
    });
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    expect(wrapper.text()).not.toContain("Score système");
  });

  it("renders a disk usage tile for the root (/) mountpoint", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Disque (/)"));
    const diskCard = wrapper.findAll(".nx-card").find((c) => c.text().includes("Disque (/)"))!;
    expect(diskCard.text()).toContain("20%");
  });

  it("shows no exceeded-threshold indicator when usage stays under the configured (default) thresholds", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    expect(wrapper.find(".nx-stat-tile__dot--danger").exists()).toBe(false);
  });

  it("shows an exceeded-threshold indicator on CPU/RAM/disk tiles once usage crosses the user's configured thresholds", async () => {
    // Mock usage: CPU 12.5%, RAM 50%, disk (/) 20% -- all comfortably under
    // the defaults (80/80/85), so dropping the thresholds below each of
    // those values is what must flip the indicator on, not the usage
    // itself changing.
    const preferences = usePreferencesStore();
    preferences.setCpuAlertThreshold(10);
    preferences.setRamAlertThreshold(40);
    preferences.setDiskAlertThreshold(15);

    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));

    const cpuCard = wrapper.findAll(".nx-card").find((c) => c.text().includes("Test CPU"))!;
    expect(cpuCard.find(".nx-stat-tile__dot--danger").exists()).toBe(true);
    const ramCard = wrapper.findAll(".nx-card").find((c) => c.text().includes("Mémoire"))!;
    expect(ramCard.find(".nx-stat-tile__dot--danger").exists()).toBe(true);
    const diskCard = wrapper.findAll(".nx-card").find((c) => c.text().includes("Disque (/)"))!;
    expect(diskCard.find(".nx-stat-tile__dot--danger").exists()).toBe(true);
  });

  it("shows no comparison chips on the very first visit (no snapshot saved yet)", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Score système"));
    expect(wrapper.find(".delta-chip").exists()).toBe(false);
  });

  it("compares against the previously saved snapshot: hides a sub-1% delta, shows danger for a worsened metric and success for an improved one", async () => {
    // Mock is CPU 12.5% / RAM 50% / disk(/) 20%. Disk's prior value is only
    // 0.5 points off (must stay hidden); CPU got worse (danger, red);
    // RAM got better (success, green).
    localStorage.setItem("nitrux-dashboard-snapshot", JSON.stringify({ cpu: 5, ram: 70, disk: 19.5 }));
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Score système"));

    const chips = wrapper.findAll(".delta-chip");
    expect(chips.length).toBe(2);
    const cpuChip = chips.find((c) => c.text().includes("CPU"))!;
    expect(cpuChip.text()).toContain("+7.5%");
    expect(cpuChip.classes()).toContain("danger");
    const ramChip = chips.find((c) => c.text().includes("RAM"))!;
    expect(ramChip.text()).toContain("-20.0%");
    expect(ramChip.classes()).toContain("success");
  });

  it("persists the current snapshot to localStorage for the next visit", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Score système"));
    const saved = JSON.parse(localStorage.getItem("nitrux-dashboard-snapshot")!);
    expect(saved).toEqual({ cpu: 12.5, ram: 50, disk: 20 });
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
