import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BackupPage from "./BackupPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "create_backup") return Promise.resolve(`/home/dev/nitrux-backup-1735689600.tar.gz`);
    return Promise.resolve(null);
  }),
}));

describe("BackupPage", () => {
  it("creates a backup and shows the resulting path", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(BackupPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer la sauvegarde")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux-backup-1735689600.tar.gz"));
    expect(invoke).toHaveBeenCalledWith("create_backup", { sourceDir: "/home/dev" });
  });

  it("shows an error message when backup creation fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("échec de la sauvegarde");
    const wrapper = mount(BackupPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer la sauvegarde")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("échec de la sauvegarde"));
  });
});
