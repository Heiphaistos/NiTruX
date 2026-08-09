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

  it("shows an empty-state message instead of a blank page when there are no snapshots", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementationOnce((cmd: string) =>
      cmd === "list_snapshots" ? Promise.resolve([]) : Promise.resolve(null),
    );
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun instantané trouvé."));
  });

  it("creates a new snapshot and refreshes the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer un instantané")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("create_snapshot"));
  });

  it("requires typing the exact snapshot date before the delete button is enabled", async () => {
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("2026-08-01"));
    await wrapper.findAll("button").find((b) => b.text() === "Supprimer")!.trigger("click");

    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la suppression")!;
    expect(confirmButton.attributes("disabled")).toBeDefined();

    await wrapper.find(".nx-input").setValue("wrong");
    expect(confirmButton.attributes("disabled")).toBeDefined();

    await wrapper.find(".nx-input").setValue("2026-08-01");
    expect(confirmButton.attributes("disabled")).toBeUndefined();
  });

  it("deletes a snapshot by its date (not its numeric id) and refreshes the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("2026-08-01"));
    await wrapper.findAll("button").find((b) => b.text() === "Supprimer")!.trigger("click");
    await wrapper.find(".nx-input").setValue("2026-08-01");
    await wrapper.findAll("button").find((b) => b.text() === "Confirmer la suppression")!.trigger("click");

    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("delete_snapshot", { name: "2026-08-01" }));
    // list_snapshots must be called again after a successful delete.
    const listCalls = vi.mocked(invoke).mock.calls.filter((c) => c[0] === "list_snapshots").length;
    expect(listCalls).toBeGreaterThan(1);
  });

  it("shows an error message when the delete fails, without hiding the confirmation row", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_snapshots") return Promise.resolve([{ id: "1", date: "2026-08-01" }]);
      if (cmd === "delete_snapshot") return Promise.reject("timeshift a échoué (code 1) : snapshot introuvable");
      return Promise.resolve(null);
    });
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("2026-08-01"));
    await wrapper.findAll("button").find((b) => b.text() === "Supprimer")!.trigger("click");
    await wrapper.find(".nx-input").setValue("2026-08-01");
    await wrapper.findAll("button").find((b) => b.text() === "Confirmer la suppression")!.trigger("click");

    await vi.waitFor(() => expect(wrapper.text()).toContain("snapshot introuvable"));
  });
});
