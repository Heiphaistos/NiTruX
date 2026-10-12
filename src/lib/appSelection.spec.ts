import { describe, it, expect } from "vitest";
import { serializeAppSelection, parseAppSelection } from "./appSelection";

describe("app selection files", () => {
  it("round-trips a list of catalog ids", () => {
    const parsed = parseAppSelection(serializeAppSelection(["firefox", "vlc"]));
    expect(parsed).toEqual({ ok: true, ids: ["firefox", "vlc"], unknown: 0 });
  });

  it("drops ids that are not in the catalog instead of installing them", () => {
    const parsed = parseAppSelection(JSON.stringify({ format: "nitrux-apps", version: 1, apps: ["vlc", "rm -rf /", 42, "vlc"] }));
    expect(parsed).toEqual({ ok: true, ids: ["vlc"], unknown: 3 });
  });

  it("rejects other JSON files and non-JSON", () => {
    expect(parseAppSelection("{}").ok).toBe(false);
    expect(parseAppSelection("pas du json").ok).toBe(false);
  });
});
