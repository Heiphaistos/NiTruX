import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DataRecoveryPage from "./DataRecoveryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_trash") {
      return Promise.resolve([
        { trashed_name: "report.pdf", original_path: "/home/dev/documents/report.pdf", deletion_date: "2026-08-01T14:30:00" },
      ]);
    }
    if (cmd === "restore_trash_item") return Promise.resolve(null);
    if (cmd === "delete_trash_item_permanently") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("DataRecoveryPage", () => {
  it("lists trashed items on mount", async () => {
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("report.pdf"));
    expect(wrapper.text()).toContain("/home/dev/documents/report.pdf");
  });

  it("restores an item and removes it from the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("report.pdf"));
    const restoreButton = wrapper.findAll("button").find((b) => b.text() === "Restaurer")!;
    await restoreButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("report.pdf"));
    expect(invoke).toHaveBeenCalledWith("restore_trash_item", { trashedName: "report.pdf" });
  });

  it("shows an empty-state message when the trash is empty", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce([]);
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Corbeille vide"));
  });

  it("requires typing the trashed file's name before permanent deletion is possible", async () => {
    // Regression guard for the actual gap: "Supprimer définitivement" is
    // genuinely irreversible (unlike Restaurer), but previously deleted
    // on a single click with zero confirmation -- unlike every other
    // comparably irreversible action in this app (uninstall,
    // format-partition), which all require typing an exact match first.
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("report.pdf"));

    const deleteButton = wrapper.findAll("button").find((b) => b.text() === "Supprimer définitivement")!;
    await deleteButton.trigger("click");

    // No confirm text typed yet: delete_trash_item_permanently must not
    // have been called, and the confirm button must be disabled.
    expect(invoke).not.toHaveBeenCalledWith("delete_trash_item_permanently", expect.anything());
    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la suppression")!;
    expect(confirmButton.attributes("disabled")).toBeDefined();

    await wrapper.find("input").setValue("report.pdf");
    await confirmButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("report.pdf"));
    expect(invoke).toHaveBeenCalledWith("delete_trash_item_permanently", { trashedName: "report.pdf" });
  });
});
