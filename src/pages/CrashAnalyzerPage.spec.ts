import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import CrashAnalyzerPage from "./CrashAnalyzerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_crash_events") {
      return Promise.resolve([
        { kind: "OomKill", message: "Out of memory: Killed process 4821 (chromium)", unit: "kernel" },
        { kind: "Segfault", message: "firefox[123]: segfault at 0 ip 00007f8a", unit: "kernel" },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("CrashAnalyzerPage", () => {
  it("lists crash events with their message and unit", async () => {
    const wrapper = mount(CrashAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Out of memory: Killed process 4821 (chromium)"));
    expect(wrapper.text()).toContain("kernel");
  });

  it("shows a French label distinguishing OOM kills from segfaults, not the raw enum tag", async () => {
    const wrapper = mount(CrashAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Manque de mémoire"));
    expect(wrapper.text()).toContain("Erreur de segmentation");
    expect(wrapper.text()).not.toContain("OomKill");
    expect(wrapper.text()).not.toContain("Segfault");
  });

  it("shows a reassuring empty-state message when no crash events are found", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_crash_events") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(CrashAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune panne détectée"));
  });

  it("shows a clear error instead of a blank page when get_crash_events is rejected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_crash_events") return Promise.reject("journalctl introuvable");
      return Promise.resolve(null);
    });
    const wrapper = mount(CrashAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("journalctl introuvable"));
  });

  it("marks a kernel panic with the danger status, not the milder warning used for segfaults", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_crash_events") {
        return Promise.resolve([
          { kind: "KernelPanic", message: "Kernel panic - not syncing: Fatal exception", unit: "kernel" },
        ]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(CrashAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Panique noyau"));
    const badge = wrapper.findAll(".nx-badge").find((b) => b.text() === "Panique noyau")!;
    expect(badge.classes()).toContain("nx-badge--danger");
  });
});
