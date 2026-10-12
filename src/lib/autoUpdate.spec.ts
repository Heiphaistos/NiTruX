import { describe, it, expect, vi, beforeEach } from "vitest";

const check = vi.fn();
const message = vi.fn();
vi.mock("@tauri-apps/plugin-updater", () => ({ check: () => check() }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ ask: vi.fn(), message: (...a: unknown[]) => message(...a) }));
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
});
