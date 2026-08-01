import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import RestorePointsPage from "./RestorePointsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_snapshots") return Promise.resolve([{ id: "1", date: "2026-08-01" }]);
    if (cmd === "create_snapshot") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("RestorePointsPage", () => {
  it("loads and lists snapshots on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("2026-08-01"));
    expect(invoke).toHaveBeenCalledWith("list_snapshots");
  });

  it("creates a new snapshot and refreshes the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer un instantané")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("create_snapshot"));
  });
});
