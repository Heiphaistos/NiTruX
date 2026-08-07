import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import SystemToolsPage from "./SystemToolsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "run_script" && args?.content === "uname -a") {
      return Promise.resolve("Linux DEV 6.12.0 x86_64");
    }
    if (cmd === "run_system_tool" && args?.action === "systemd-reload") {
      return Promise.resolve("");
    }
    return Promise.resolve(null);
  }),
}));

describe("SystemToolsPage", () => {
  it("runs a non-privileged tool via run_script and shows its output", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(SystemToolsPage);
    const card = wrapper.findAll(".st-card").find((c) => c.text().includes("Informations noyau"))!;
    await card.find("button").trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Linux DEV 6.12.0 x86_64"));
    expect(invoke).toHaveBeenCalledWith("run_script", { content: "uname -a" });
  });

  it("runs a privileged tool via run_system_tool, not run_script", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(SystemToolsPage);
    const card = wrapper.findAll(".st-card").find((c) => c.text().includes("Recharger systemd"))!;
    await card.find("button").trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("run_system_tool", { action: "systemd-reload" }));
  });

  it("shows a root badge on privileged tools", () => {
    const wrapper = mount(SystemToolsPage);
    const card = wrapper.findAll(".st-card").find((c) => c.text().includes("Recharger systemd"))!;
    expect(card.text()).toContain("root");
  });

  it("filters the catalog by category", async () => {
    const wrapper = mount(SystemToolsPage);
    const chips = wrapper.findAll(".st-chip");
    const networkChip = chips.find((c) => c.text() === "Réseau")!;
    await networkChip.trigger("click");
    expect(wrapper.text()).toContain("Test de connectivité");
    expect(wrapper.text()).not.toContain("Informations noyau");
  });

  it("filters the catalog by search text", async () => {
    const wrapper = mount(SystemToolsPage);
    const search = wrapper.find("input");
    await search.setValue("DNS");
    expect(wrapper.text()).toContain("Résolution DNS");
    expect(wrapper.text()).not.toContain("Informations noyau");
  });

  it("shows an empty-state message when a search matches no tool", async () => {
    const wrapper = mount(SystemToolsPage);
    const search = wrapper.find("input");
    await search.setValue("zzznotarealtoolzzz");
    expect(wrapper.text()).toContain("Aucun outil ne correspond à cette recherche.");
  });

  it("keeps a tool's button disabled while it is running, even after a different tool starts running too", async () => {
    // `running` was a single ref holding at most one tool id -- clicking a
    // second, different tool while the first is still in flight overwrites
    // it, so the first tool's button re-evaluates `running === tool.id` to
    // false and re-enables while genuinely still running (its own `finally`
    // hasn't fired yet). A user could then click it again, invoking the
    // same command a second time concurrently. Each tool's `outputs`/
    // `errors` are already tracked per-id (Record<string, ...>); `running`
    // must be too.
    const { invoke } = await import("@tauri-apps/api/core");
    let resolveFirst!: (v: string) => void;
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "run_script" && args?.content === "uname -a") {
        return new Promise<string>((resolve) => { resolveFirst = resolve; });
      }
      if (cmd === "run_system_tool" && args?.action === "systemd-reload") {
        return new Promise<string>(() => {}); // never resolves within this test
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(SystemToolsPage);
    const firstCard = wrapper.findAll(".st-card").find((c) => c.text().includes("Informations noyau"))!;
    await firstCard.find("button").trigger("click");
    expect(firstCard.find("button").attributes("disabled")).toBeDefined();

    const secondCard = wrapper.findAll(".st-card").find((c) => c.text().includes("Recharger systemd"))!;
    await secondCard.find("button").trigger("click");

    expect(firstCard.find("button").attributes("disabled")).toBeDefined();
    resolveFirst("Linux DEV 6.12.0 x86_64");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Linux DEV 6.12.0 x86_64"));
  });

  it("clears a previous run's stale output when a rerun of the same tool fails, instead of showing both at once", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "run_script" && args?.content === "uname -a") return Promise.resolve("Linux DEV 6.12.0 x86_64");
      return Promise.resolve(null);
    });
    const wrapper = mount(SystemToolsPage);
    const card = wrapper.findAll(".st-card").find((c) => c.text().includes("Informations noyau"))!;
    await card.find("button").trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Linux DEV 6.12.0 x86_64"));

    // Rerun the same tool, this time it fails.
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "run_script" && args?.content === "uname -a") return Promise.reject("commande introuvable");
      return Promise.resolve(null);
    });
    await card.find("button").trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("commande introuvable"));

    expect(wrapper.text()).not.toContain("Linux DEV 6.12.0 x86_64");
  });
});
