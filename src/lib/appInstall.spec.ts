import { describe, it, expect, vi, beforeEach } from "vitest";
import { installWithFallback, exactMatches, describeFailure, type SearchResult } from "./appInstall";

// A plain function rather than vi.fn(): the spy's own result tracking
// reported the (handled) rejections these tests are about as failures.
type Impl = (cmd: string, args: Record<string, unknown>) => Promise<unknown>;
const fake = vi.hoisted(() => ({ impl: (async () => null) as unknown as Impl }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (cmd: string, args: Record<string, unknown>) => fake.impl(cmd, args) }));
const invoke = { mockImplementation: (f: Impl) => { fake.impl = f; } };

const r = (source: SearchResult["source"], id: string, name: string, manager: string | null = null): SearchResult => ({
  source, id, name, manager, summary: "", version: "",
});

describe("appInstall", () => {
  beforeEach(() => { fake.impl = async () => null; });

  it("keeps only exact matches, native first, then Flatpak, then Snap", () => {
    const list = [
      r("snap", "vlc", "vlc"),
      r("flatpak", "org.videolan.VLC", "VLC"),
      r("native", "vlc-plugin-base", "vlc-plugin-base", "apt"),
      r("native", "vlc", "vlc", "dnf"),
    ];
    expect(exactMatches(list, "VLC", "vlc").map((c) => c.source)).toEqual(["native", "flatpak", "snap"]);
  });

  it("falls back to Flathub when the distribution has no package under the catalog's name", async () => {
    invoke.mockImplementation((cmd, args) => {
      if (cmd === "install_package") return Promise.reject(`paquet introuvable : ${args.package}`);
      if (cmd === "search_packages") return Promise.resolve({ results: [r("flatpak", "org.mozilla.firefox", "Firefox")], errors: [] });
      if (cmd === "install_flatpak_package") return Promise.resolve("ok");
      return Promise.resolve(null);
    });
    const res = await installWithFallback(
      [{ source: "native", id: "firefox-esr", manager: "dnf" }, { source: "native", id: "firefox", manager: "dnf" }],
      { name: "Firefox", entryId: "firefox" },
    );
    expect(res.ok).toBe(true);
    if (res.ok) expect(res.used).toEqual({ source: "flatpak", id: "org.mozilla.firefox", manager: null });
    expect(res.attempts.filter((a) => !a.ok)).toHaveLength(2);
  });

  it("never retries a candidate the search returns again", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "search_packages") return { results: [r("native", "gimp", "gimp", "apt")], errors: [] };
      throw new Error("échec");
    });
    const res = await installWithFallback([{ source: "native", id: "gimp", manager: "apt" }], { name: "GIMP", entryId: "gimp" });
    expect(res.ok).toBe(false);
    expect(res.attempts).toHaveLength(1);
    expect(describeFailure(res.attempts)).toMatch(/^apt \(gimp\) : .*échec/);
  });

  it("tries the Arch name first on pacman, and never repeats a name", async () => {
    const { catalogCandidates } = await import("./appInstall");
    const entry = { id: "libreoffice", name: "LibreOffice", description: "", icon: "", category: "", installMethod: "apt" as const, packageId: "libreoffice", arch: "libreoffice-fresh" };
    expect(catalogCandidates(entry, "pacman").map((c) => c.id)).toEqual(["libreoffice-fresh", "libreoffice"]);
    expect(catalogCandidates(entry, "apt").map((c) => c.id)).toEqual(["libreoffice"]);
    expect(catalogCandidates({ ...entry, installMethod: "flatpak", packageId: "org.libreoffice.LibreOffice" }, "pacman")).toEqual([
      { source: "flatpak", id: "org.libreoffice.LibreOffice" },
    ]);
    expect(catalogCandidates(entry, null)).toEqual([]);
  });

  it("points to the AUR when every pacman attempt failed", async () => {
    const { describeFailure } = await import("./appInstall");
    const text = describeFailure([{ candidate: { source: "native", id: "pidgin", manager: "pacman" }, ok: false, message: "introuvable" }]);
    expect(text).toContain("yay -S pidgin");
  });
});
