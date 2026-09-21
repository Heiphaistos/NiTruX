import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import BackupPage from "./BackupPage.vue";

const defaultInvokeImpl = vi.hoisted(() => (cmd: string) => {
  if (cmd === "create_backup") return Promise.resolve(`/home/dev/nitrux-backup-1735689600.tar.gz`);
  if (cmd === "list_backups") {
    return Promise.resolve([
      { path: "/home/dev/nitrux-backup-1735689600.tar.gz", size_bytes: 52_428_800, created_epoch_secs: 1_735_689_600 },
    ]);
  }
  return Promise.resolve(null);
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(defaultInvokeImpl) }));

describe("BackupPage", () => {
  beforeEach(async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation(defaultInvokeImpl);
  });

  it("lists the backups already present in the home directory", async () => {
    const wrapper = mount(BackupPage);
    // Wait on the row itself, not the static section header: the header
    // renders immediately and the list arrives one tick later.
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux-backup-1735689600.tar.gz"));
    expect(wrapper.text()).toContain("50.0 Mo");
  });

  it("says so when no backup exists yet", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "list_backups" ? Promise.resolve([]) : defaultInvokeImpl(cmd),
    );
    const wrapper = mount(BackupPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune sauvegarde créée par NiTruX"));
  });

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
    // Targeted at create_backup, not "the next call": the page now also
    // calls list_backups on mount, which would otherwise eat the rejection.
    vi.mocked(invoke).mockImplementation((cmd: string) =>
      cmd === "create_backup" ? Promise.reject("échec de la sauvegarde") : defaultInvokeImpl(cmd),
    );
    const wrapper = mount(BackupPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer la sauvegarde")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("échec de la sauvegarde"));
  });
});
