import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useProfilesStore } from "./profilesStore";
import { useThemeStore } from "./themeStore";
import { useLayoutStore } from "./layoutStore";
import { useStyleStore } from "./styleStore";
import { usePreferencesStore } from "./preferencesStore";
import { builtinThemes } from "@/themes/builtin";

describe("profilesStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.documentElement.removeAttribute("style");
    localStorage.clear();
  });

  it("starts with no saved profiles", () => {
    expect(useProfilesStore().profiles).toEqual([]);
  });

  it("saves the current theme/layout/style/preferences as a named profile", () => {
    const theme = useThemeStore();
    const layout = useLayoutStore();
    const style = useStyleStore();
    const preferences = usePreferencesStore();
    theme.setTheme(builtinThemes[1]);
    layout.setLayout("sidebar-classic");
    style.setStyle("neon-terminal");
    preferences.setDefaultScanDirectory("/home/dev/Downloads");

    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");

    expect(profiles.profiles.length).toBe(1);
    const saved = profiles.profiles[0];
    expect(saved.name).toBe("Bureau");
    expect(saved.theme.id).toBe(builtinThemes[1].id);
    expect(saved.layout).toBe("sidebar-classic");
    expect(saved.style).toBe("neon-terminal");
    expect(saved.preferences.defaultScanDirectory).toBe("/home/dev/Downloads");
  });

  it("persists profiles to localStorage and restores them on next init", () => {
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");

    setActivePinia(createPinia());
    const reloaded = useProfilesStore();
    expect(reloaded.profiles.map((p) => p.name)).toContain("Bureau");
  });

  it("a saved profile is a snapshot, not a live reference -- later changes don't retroactively alter it", () => {
    const theme = useThemeStore();
    theme.setTheme(builtinThemes[0]);
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Snapshot");

    theme.setTheme(builtinThemes[1]); // change the active theme AFTER saving

    expect(profiles.profiles[0].theme.id).toBe(builtinThemes[0].id);
  });

  it("applying a profile restores theme, layout, style, and preferences", () => {
    const theme = useThemeStore();
    const layout = useLayoutStore();
    const style = useStyleStore();
    const preferences = usePreferencesStore();
    theme.setTheme(builtinThemes[1]);
    layout.setLayout("sidebar-classic");
    style.setStyle("neon-terminal");
    preferences.setDefaultScanDirectory("/home/dev/Downloads");

    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");

    // Switch everything to something else before applying, to prove apply()
    // actually restores state rather than the assertions trivially passing
    // because nothing changed.
    theme.setTheme(builtinThemes[0]);
    layout.setLayout("master-detail");
    style.setStyle("glass-glow");
    preferences.setDefaultScanDirectory("/tmp");

    profiles.apply(profiles.profiles[0]);

    expect(theme.active.id).toBe(builtinThemes[1].id);
    expect(layout.current).toBe("sidebar-classic");
    expect(style.current).toBe("neon-terminal");
    expect(preferences.defaultScanDirectory).toBe("/home/dev/Downloads");
  });

  it("applying a profile also restores the CPU/RAM/disk alert thresholds", () => {
    // Regression guard for the actual bug: saveCurrentAs captures every
    // field of preferencesStore's state via a full spread (including the
    // three alert thresholds), but apply() only ever restored
    // defaultScanDirectory/dashboardRefreshIntervalMs -- "Enregistrer"
    // silently kept the thresholds while "Appliquer" silently dropped them.
    const preferences = usePreferencesStore();
    preferences.setCpuAlertThreshold(65);
    preferences.setRamAlertThreshold(70);
    preferences.setDiskAlertThreshold(90);

    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Seuils personnalisés");

    // Change every threshold away from the saved values before applying.
    preferences.setCpuAlertThreshold(80);
    preferences.setRamAlertThreshold(80);
    preferences.setDiskAlertThreshold(85);

    profiles.apply(profiles.profiles[0]);

    expect(preferences.cpuAlertThreshold).toBe(65);
    expect(preferences.ramAlertThreshold).toBe(70);
    expect(preferences.diskAlertThreshold).toBe(90);
  });

  it("removes a profile", () => {
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");
    const id = profiles.profiles[0].id;
    profiles.remove(id);
    expect(profiles.profiles).toEqual([]);
  });

  it("exports a profile as JSON and re-imports it under a fresh id", () => {
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Bureau");
    const original = profiles.profiles[0];
    const json = profiles.exportProfile(original);

    const result = profiles.importProfile(json);

    expect(result.ok).toBe(true);
    expect(profiles.profiles.length).toBe(2);
    const imported = profiles.profiles[1];
    expect(imported.name).toBe("Bureau");
    expect(imported.id).not.toBe(original.id);
  });

  it("rejects an imported profile with a missing or invalid field", () => {
    const profiles = useProfilesStore();
    const incomplete = { id: "x", name: "Broken", theme: builtinThemes[0], layout: "master-detail" };
    // preferences field missing entirely
    const result = profiles.importProfile(JSON.stringify(incomplete));
    expect(result.ok).toBe(false);
    expect(profiles.profiles).toEqual([]);
  });

  it("rejects an imported profile whose preferences object is missing an alert threshold field", () => {
    // Regression guard: isValidPreferences used to check only
    // defaultScanDirectory/dashboardRefreshIntervalMs, out of sync with the
    // real 5-field Preferences interface -- a preferences object missing
    // cpuAlertThreshold/ramAlertThreshold/diskAlertThreshold used to pass
    // validation and get imported anyway.
    const profiles = useProfilesStore();
    profiles.saveCurrentAs("Valide");
    const withoutThresholds = {
      ...profiles.profiles[0],
      preferences: { defaultScanDirectory: "/tmp", dashboardRefreshIntervalMs: 2000 },
    };
    const result = profiles.importProfile(JSON.stringify(withoutThresholds));
    expect(result.ok).toBe(false);
    expect(profiles.profiles.length).toBe(1); // only the original valid one
  });

  it("rejects malformed JSON", () => {
    const profiles = useProfilesStore();
    const result = profiles.importProfile("not valid json{");
    expect(result.ok).toBe(false);
    expect(profiles.profiles).toEqual([]);
  });
});
