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

  it("includes the profiles added to broaden coverage beyond the original 4 (jeux, sécurité, serveur web)", () => {
    // The generic invariant checks above would still pass even if these
    // were accidentally deleted -- this pins their presence so that
    // regression is actually caught.
    const ids = installProfiles.map((p) => p.id);
    expect(ids).toEqual(expect.arrayContaining(["jeux", "securite", "serveur-web"]));
  });

  it("includes the profiles added for science/education and accessibility coverage", () => {
    const ids = installProfiles.map((p) => p.id);
    expect(ids).toEqual(expect.arrayContaining(["science-education", "accessibilite"]));
  });

  it("'developpement' profile has real dev tooling, not the miscategorized 'htop' (a Utilitaires entry)", () => {
    // Regression guard for the actual bug: this profile shipped with only
    // ["git", "htop"] -- htop's own appCatalog entry is category
    // "Utilitaires", not "Développement", so it never belonged here, and
    // 2 apps was a token gesture against a catalog with 60+ real dev tools.
    const profile = installProfiles.find((p) => p.id === "developpement")!;
    expect(profile.appIds).not.toContain("htop");
    expect(profile.appIds).toEqual(expect.arrayContaining(["git", "vscode", "build-essential", "python3", "nodejs"]));
  });

  it("includes the new 'virtualisation' profile covering a previously-unaddressed catalog category", () => {
    // "Virtualisation" has 6 real appCatalog entries (qemu-system,
    // virt-manager, docker, docker-compose, podman, gnome-boxes) but no
    // profile drew from it at all before this -- a genuine coverage gap,
    // distinct from "developpement" which is coding tools, not VM/containers.
    const ids = installProfiles.map((p) => p.id);
    expect(ids).toContain("virtualisation");
    const profile = installProfiles.find((p) => p.id === "virtualisation")!;
    expect(profile.appIds).toEqual(expect.arrayContaining(["qemu-system", "virt-manager", "docker", "docker-compose"]));
  });

  it("includes the new 'cloud-sync' profile, the entire small 'Cloud & Synchronisation' catalog category", () => {
    // Only 5 real appCatalog entries exist under this category and none
    // were covered by any profile -- small enough to include the category
    // whole, unlike the larger categories (Développement, Jeux) that need
    // curation down to a representative subset.
    const ids = installProfiles.map((p) => p.id);
    expect(ids).toContain("cloud-sync");
    const profile = installProfiles.find((p) => p.id === "cloud-sync")!;
    expect(profile.appIds).toEqual(
      expect.arrayContaining(["nextcloud-desktop", "dropbox", "syncthing", "davfs2", "gigolo"])
    );
  });
});
