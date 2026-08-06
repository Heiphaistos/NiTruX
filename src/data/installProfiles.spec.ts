// src/data/installProfiles.spec.ts
import { describe, it, expect } from "vitest";
import { installProfiles } from "./installProfiles";
import { appCatalog } from "./appCatalog";

describe("installProfiles", () => {
  it("has no duplicate ids", () => {
    const ids = installProfiles.map((p) => p.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every entry has non-empty required fields and at least one app", () => {
    for (const profile of installProfiles) {
      expect(profile.id.length).toBeGreaterThan(0);
      expect(profile.label.length).toBeGreaterThan(0);
      expect(profile.description.length).toBeGreaterThan(0);
      expect(profile.appIds.length).toBeGreaterThan(0);
    }
  });

  // Every appId here must resolve to a real appCatalog entry -- otherwise
  // InstallProfilesPage.vue's selectProfile()/selectedEntries silently
  // drops the unresolved id (catalogById.get(id) -> undefined, filtered
  // out) with no error shown anywhere, while the profile button still
  // displays the ORIGINAL, now-wrong count ("{{ profile.appIds.length }}
  // applications"). Manually verified once (2026-08-06, cycle 136) that
  // all appIds resolved at the time; codifying it here so a future
  // profile addition/typo, or an appCatalog entry being renamed/removed,
  // is caught immediately instead of silently under-installing.
  it("references only appIds that actually exist in appCatalog", () => {
    const catalogIds = new Set(appCatalog.map((e) => e.id));
    for (const profile of installProfiles) {
      for (const appId of profile.appIds) {
        expect(catalogIds.has(appId), `${profile.id} references unknown appId "${appId}"`).toBe(true);
      }
    }
  });

  it("never lists the same appId twice within one profile", () => {
    for (const profile of installProfiles) {
      expect(new Set(profile.appIds).size, `${profile.id} has a duplicate appId`).toBe(profile.appIds.length);
    }
  });
});
