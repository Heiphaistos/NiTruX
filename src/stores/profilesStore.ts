import { defineStore } from "pinia";
import { useThemeStore } from "./themeStore";
import { useLayoutStore } from "./layoutStore";
import { useStyleStore } from "./styleStore";
import { usePreferencesStore, type Preferences } from "./preferencesStore";
import { REQUIRED_COLOR_KEYS } from "./themeStore";
import type { Theme } from "@/types/theme";
import type { LayoutId } from "@/types/layout";
import type { StyleId } from "@/types/style";
import { layoutRegistry } from "@/layouts/registry";
import { styleRegistry } from "@/styles/registry";

export interface ConfigProfile {
  id: string;
  name: string;
  theme: Theme;
  layout: LayoutId;
  style: StyleId;
  preferences: Preferences;
}

const STORAGE_KEY = "nitrux-config-profiles";

function hasAllRequiredColors(colors: unknown): colors is Theme["colors"] {
  if (typeof colors !== "object" || colors === null) return false;
  return REQUIRED_COLOR_KEYS.every((key) => key in (colors as Record<string, unknown>));
}

function isValidLayoutId(value: unknown): value is LayoutId {
  return typeof value === "string" && layoutRegistry.some((l) => l.id === value);
}

function isValidStyleId(value: unknown): value is StyleId {
  return typeof value === "string" && styleRegistry.some((s) => s.id === value);
}

function isValidPreferences(value: unknown): value is Preferences {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.defaultScanDirectory === "string" &&
    typeof v.dashboardRefreshIntervalMs === "number" &&
    typeof v.cpuAlertThreshold === "number" &&
    typeof v.ramAlertThreshold === "number" &&
    typeof v.diskAlertThreshold === "number"
  );
}

function isConfigProfile(value: unknown): value is ConfigProfile {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.id === "string" &&
    typeof v.name === "string" &&
    typeof v.theme === "object" &&
    v.theme !== null &&
    typeof (v.theme as Theme).id === "string" &&
    hasAllRequiredColors((v.theme as Theme).colors) &&
    isValidLayoutId(v.layout) &&
    isValidStyleId(v.style) &&
    isValidPreferences(v.preferences)
  );
}

function readPersistedProfiles(): ConfigProfile[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return [];
  try {
    const parsed = JSON.parse(stored) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isConfigProfile);
  } catch {
    return [];
  }
}

function persist(profiles: ConfigProfile[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles));
}

export const useProfilesStore = defineStore("profiles", {
  state: () => ({
    profiles: readPersistedProfiles(),
  }),
  actions: {
    // Snapshots the theme/layout/style/preferences stores' CURRENT state as
    // a new named profile -- not a live reference, so later changes to the
    // active theme etc. never retroactively mutate a saved profile.
    saveCurrentAs(name: string): void {
      const themeStore = useThemeStore();
      const layoutStore = useLayoutStore();
      const styleStore = useStyleStore();
      const preferencesStore = usePreferencesStore();
      const profile: ConfigProfile = {
        // Found live by this file's own test: `profile-${Date.now()}` (the
        // pattern used elsewhere in this codebase, e.g. ThemeEditorPage's
        // custom theme ids) collides when two profiles are created in the
        // same millisecond -- realistic here since a save and an import can
        // both fire in the same test/batch tick, not just a hypothetical.
        id: crypto.randomUUID(),
        name,
        theme: { ...themeStore.active, colors: { ...themeStore.active.colors } },
        layout: layoutStore.current,
        style: styleStore.current,
        preferences: { ...preferencesStore.$state },
      };
      this.profiles.push(profile);
      persist(this.profiles);
    },
    apply(profile: ConfigProfile): void {
      const themeStore = useThemeStore();
      const layoutStore = useLayoutStore();
      const styleStore = useStyleStore();
      const preferencesStore = usePreferencesStore();
      themeStore.setTheme(profile.theme);
      layoutStore.setLayout(profile.layout);
      styleStore.setStyle(profile.style);
      preferencesStore.setDefaultScanDirectory(profile.preferences.defaultScanDirectory);
      preferencesStore.setDashboardRefreshIntervalMs(profile.preferences.dashboardRefreshIntervalMs);
      // The three alert thresholds are captured by saveCurrentAs's full
      // `{ ...preferencesStore.$state }` spread but were never restored
      // here -- "Enregistrer" silently captured them while "Appliquer"
      // silently dropped them, so switching profiles never actually
      // switched a user's CPU/RAM/disk alert thresholds despite the page's
      // own description promising the full "préférences actuelles".
      preferencesStore.setCpuAlertThreshold(profile.preferences.cpuAlertThreshold);
      preferencesStore.setRamAlertThreshold(profile.preferences.ramAlertThreshold);
      preferencesStore.setDiskAlertThreshold(profile.preferences.diskAlertThreshold);
    },
    remove(id: string): void {
      this.profiles = this.profiles.filter((p) => p.id !== id);
      persist(this.profiles);
    },
    exportProfile(profile: ConfigProfile): string {
      return JSON.stringify(profile, null, 2);
    },
    importProfile(json: string): { ok: true } | { ok: false; error: string } {
      let parsed: unknown;
      try {
        parsed = JSON.parse(json);
      } catch {
        return { ok: false, error: "JSON invalide." };
      }
      if (!isConfigProfile(parsed)) {
        return { ok: false, error: "Fichier de profil invalide : champs manquants ou incorrects." };
      }
      // A fresh id avoids silently colliding with (and overwriting) an
      // existing profile that happens to share the imported one's id --
      // mirrors themeStore.importTheme's own choice to always persist as a
      // new entry rather than trust an id minted on a different machine.
      const profile: ConfigProfile = { ...parsed, id: crypto.randomUUID() };
      this.profiles.push(profile);
      persist(this.profiles);
      return { ok: true };
    },
  },
});
