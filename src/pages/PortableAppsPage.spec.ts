import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import PortableAppsPage from "./PortableAppsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "list_portable_apps") return Promise.resolve([]);
    if (cmd === "download_portable_app") {
      if (args?.repo === "fail-repo") return Promise.reject("aucun fichier AppImage trouvé");
      return Promise.resolve({ filename: "Joplin-3.6.15.AppImage", size_bytes: 123456789 });
    }
    if (cmd === "launch_portable_app") return Promise.resolve(null);
    if (cmd === "remove_portable_app") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("PortableAppsPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("lists the catalog on mount and shows an empty downloaded-apps state", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(PortableAppsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Joplin"));
    expect(invoke).toHaveBeenCalledWith("list_portable_apps");
    // No "Applications téléchargées" section renders when nothing is downloaded yet.
    expect(wrapper.text()).not.toContain("Applications téléchargées");
  });

  it("downloads a catalog app via download_portable_app with its owner/repo, then refreshes the downloaded list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_portable_apps") return Promise.resolve([]);
      if (cmd === "download_portable_app") return Promise.resolve({ filename: "Joplin-3.6.15.AppImage", size_bytes: 123456789 });
      return Promise.resolve(null);
    });
    const wrapper = mount(PortableAppsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Joplin"));
    const joplinCard = wrapper.findAll(".pa-card").find((c) => c.text().includes("Joplin"))!;
    await joplinCard.find("button").trigger("click");
    expect(invoke).toHaveBeenCalledWith("download_portable_app", { owner: "laurent22", repo: "joplin" });
  });

  it("shows an inline error (not a blocking native alert) when a download fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "list_portable_apps") return Promise.resolve([]);
      if (cmd === "download_portable_app" && args?.owner === "laurent22") {
        return Promise.reject("aucun fichier AppImage trouvé dans la dernière release de laurent22/joplin");
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(PortableAppsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Joplin"));
    const joplinCard = wrapper.findAll(".pa-card").find((c) => c.text().includes("Joplin"))!;
    await joplinCard.find("button").trigger("click");
    await vi.waitFor(() => expect(joplinCard.text()).toMatch(/aucun fichier AppImage trouvé/));
  });

  it("lists a downloaded app with Lancer/Supprimer actions", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_portable_apps") {
        return Promise.resolve([{ filename: "Joplin-3.6.15.AppImage", size_bytes: 104857600 }]);
      }
      return Promise.resolve(null);
    });
    const wrapper = mount(PortableAppsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Applications téléchargées"));
    expect(wrapper.text()).toContain("Joplin-3.6.15.AppImage");
    expect(wrapper.text()).toContain("100.0 Mo");

    const launchButton = wrapper.findAll("button").find((b) => b.text() === "Lancer")!;
    await launchButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("launch_portable_app", { filename: "Joplin-3.6.15.AppImage" });

    const removeButton = wrapper.findAll("button").find((b) => b.text() === "Supprimer")!;
    await removeButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("remove_portable_app", { filename: "Joplin-3.6.15.AppImage" });
  });
});
