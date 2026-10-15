import { describe, it, expect, vi, beforeEach } from "vitest";

const check = vi.fn();
const message = vi.fn();
const ask = vi.fn();
const relaunch = vi.fn();
vi.mock("@tauri-apps/plugin-updater", () => ({ check: () => check() }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: () => relaunch() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ ask: (...a: unknown[]) => ask(...a), message: (...a: unknown[]) => message(...a) }));
const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe("checkForUpdate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubEnv("DEV", false);
  });

  it("leaves a pacman-managed copy to pacman and never asks the Tauri updater", async () => {
    invoke.mockResolvedValue({ managed_by: "pacman", distro_id: "omarchy" });
    const { checkForUpdate } = await import("./autoUpdate");
    expect(await checkForUpdate(false)).toBe(false);
    expect(check).not.toHaveBeenCalled();
    expect(String(message.mock.calls[0]?.[0])).toContain("omarchy-update");
  });

  it("uses the Tauri updater for deb/rpm/AppImage copies", async () => {
    invoke.mockResolvedValue({ managed_by: null, distro_id: "debian" });
    check.mockResolvedValue(null);
    const { checkForUpdate } = await import("./autoUpdate");
    await checkForUpdate(true);
    expect(check).toHaveBeenCalled();
  });

  it("says nothing when silent and up to date, answers when asked", async () => {
    invoke.mockResolvedValue({ managed_by: null, distro_id: "debian" });
    check.mockResolvedValue(null);
    const { checkForUpdate } = await import("./autoUpdate");
    await checkForUpdate(true);
    expect(message).not.toHaveBeenCalled();
    await checkForUpdate(false);
    expect(String(message.mock.calls[0]?.[0])).toContain("à jour");
  });

  it("installs and relaunches only after the user agrees", async () => {
    invoke.mockResolvedValue({ managed_by: null, distro_id: "debian" });
    const downloadAndInstall = vi.fn();
    check.mockResolvedValue({ version: "9.9.9", currentVersion: "0.26.0", body: "", downloadAndInstall });
    const { checkForUpdate } = await import("./autoUpdate");
    ask.mockResolvedValueOnce(false);
    expect(await checkForUpdate(true)).toBe(false);
    expect(downloadAndInstall).not.toHaveBeenCalled();
    ask.mockResolvedValueOnce(true);
    expect(await checkForUpdate(true)).toBe(true);
    expect(downloadAndInstall).toHaveBeenCalled();
    expect(relaunch).toHaveBeenCalled();
  });

  it("stays silent on network failure at startup, reports it on manual check", async () => {
    invoke.mockResolvedValue({ managed_by: null, distro_id: "debian" });
    check.mockRejectedValue(new Error("offline"));
    vi.spyOn(console, "error").mockImplementation(() => {});
    const { checkForUpdate } = await import("./autoUpdate");
    await checkForUpdate(true);
    expect(message).not.toHaveBeenCalled();
    await checkForUpdate(false);
    expect(String(message.mock.calls[0]?.[0])).toContain("impossible");
  });
});
